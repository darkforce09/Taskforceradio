// The extern "C" glue between the Rust oracle wrappers (src/oracles/oracle_bindings.rs) and the
// original code: DSPFilters and simpleSource from reference/tfar_1_0/ts/include/, and FAudio's
// F3DAudio vendored in oracle/.
//
// Every function takes plain structs and caller-owned buffers, catches every C++ exception and
// returns a status code, so nothing unwinds into Rust. The filter and compressor paths follow the
// originals' call sites: SimpleFilter<Design, Channels> (DirectFormII) set up as
// reference/tfar_1_0/ts/src/clientData.hpp and RadioEffect.hpp do, the stereo i16 conversion of
// helpers.hpp processFilterStereo, and SimpleComp set up and run as clientData.hpp and
// RadioEffect.hpp processCompressor do. The spatial path calls F3DAudioCalculate with the
// listener, emitter, curve and cone of helpers.cpp applyILD.

#include <array>
#include <climits>
#include <cmath>
#include <complex>
#include <cstddef>
#include <cstdint>
#include <exception>
#include <memory>
#include <vector>

#include "DspFilters/Butterworth.h"
#include "DspFilters/Filter.h"
#include "DspFilters/RBJ.h"
#include "F3DAudio.h"
#include "SimpleComp.h"

namespace {

// Status codes, mirrored by OracleStatus in src/oracles/oracle_bindings.rs.
constexpr int32_t STATUS_OK = 0;
constexpr int32_t STATUS_INVALID_ARGUMENT = 1;
constexpr int32_t STATUS_EXCEPTION = 2;

// Filter kinds, mirrored by the Rust bindings.
constexpr int32_t KIND_BUTTERWORTH_LOW_PASS = 0;
constexpr int32_t KIND_BUTTERWORTH_BAND_PASS = 1;
constexpr int32_t KIND_RBJ_LOW_PASS = 2;
constexpr int32_t KIND_RBJ_HIGH_PASS = 3;

// The most curve points a scene carries.
constexpr uint32_t MAX_CURVE_POINTS = 32;

// Failed F3DAudio parameter checks since the last reset, per thread (FAudio_internal.h).
thread_local uint32_t assertion_failures = 0;

}  // namespace

extern "C" {

// One filter design, as the Rust bindings lay it out.
struct oracle_filter_design {
    int32_t kind;
    int32_t order;
    double sample_rate_hz;
    double frequency_hz;
    double width_hz;
    double quality;
};

// The settings of one SimpleComp.
struct oracle_compressor_settings {
    double sample_rate_hz;
    double threshold_db;
    double ratio;
    double attack_ms;
    double release_ms;
};

// A vector of three floats, F3DAudio's layout.
struct oracle_vector {
    float x;
    float y;
    float z;
};

// One listener and one mono emitter, with an optional volume curve and an optional cone.
struct oracle_spatial_scene {
    oracle_vector listener_position;
    oracle_vector listener_front;
    oracle_vector listener_top;
    oracle_vector emitter_position;
    oracle_vector emitter_front;
    oracle_vector emitter_top;
    float curve_distances[MAX_CURVE_POINTS];
    float curve_gains[MAX_CURVE_POINTS];
    uint32_t curve_point_count;
    float curve_distance_scaler;
    int32_t has_cone;
    float cone_inner_angle;
    float cone_outer_angle;
    float cone_inner_volume;
    float cone_outer_volume;
};

// What F3DAudioCalculate returns for a scene.
struct oracle_spatial_result {
    float left;
    float right;
    float volume;
    float emitter_to_listener_angle;
    float emitter_to_listener_distance;
    uint32_t assertion_failures;
};

// The size of every struct shared with Rust, so the bindings' layout tests can check theirs.
struct oracle_struct_sizes {
    size_t filter_design;
    size_t compressor_settings;
    size_t vector;
    size_t spatial_scene;
    size_t spatial_result;
};

oracle_struct_sizes oracle_shim_struct_sizes(void) {
    oracle_struct_sizes sizes;
    sizes.filter_design = sizeof(oracle_filter_design);
    sizes.compressor_settings = sizeof(oracle_compressor_settings);
    sizes.vector = sizeof(oracle_vector);
    sizes.spatial_scene = sizeof(oracle_spatial_scene);
    sizes.spatial_result = sizeof(oracle_spatial_result);
    return sizes;
}

void oracle_shim_record_assertion(int holds) {
    if (holds == 0) {
        assertion_failures += 1;
    }
}

}  // extern "C"

namespace {

// Builds the SimpleFilter of `design` with `Channels` channels, sets it up and hands it to
// `visit`. Butterworth orders are template arguments, so the order picks the instantiation.
template <int Channels, int Order, typename Visit>
int32_t with_butterworth(const oracle_filter_design& design, Visit& visit) {
    if (design.kind == KIND_BUTTERWORTH_LOW_PASS) {
        auto filter =
            std::make_unique<Dsp::SimpleFilter<Dsp::Butterworth::LowPass<Order>, Channels>>();
        filter->setup(Order, design.sample_rate_hz, design.frequency_hz);
        return visit(*filter);
    }
    auto filter =
        std::make_unique<Dsp::SimpleFilter<Dsp::Butterworth::BandPass<Order>, Channels>>();
    filter->setup(Order, design.sample_rate_hz, design.frequency_hz, design.width_hz);
    return visit(*filter);
}

template <int Channels, typename Visit>
int32_t with_filter(const oracle_filter_design& design, Visit visit) {
    switch (design.kind) {
        case KIND_BUTTERWORTH_LOW_PASS:
        case KIND_BUTTERWORTH_BAND_PASS:
            switch (design.order) {
                case 1: return with_butterworth<Channels, 1>(design, visit);
                case 2: return with_butterworth<Channels, 2>(design, visit);
                case 3: return with_butterworth<Channels, 3>(design, visit);
                case 4: return with_butterworth<Channels, 4>(design, visit);
                case 5: return with_butterworth<Channels, 5>(design, visit);
                case 6: return with_butterworth<Channels, 6>(design, visit);
                case 7: return with_butterworth<Channels, 7>(design, visit);
                case 8: return with_butterworth<Channels, 8>(design, visit);
                default: return STATUS_INVALID_ARGUMENT;
            }
        case KIND_RBJ_LOW_PASS: {
            auto filter = std::make_unique<Dsp::SimpleFilter<Dsp::RBJ::LowPass, Channels>>();
            filter->setup(design.sample_rate_hz, design.frequency_hz, design.quality);
            return visit(*filter);
        }
        case KIND_RBJ_HIGH_PASS: {
            auto filter = std::make_unique<Dsp::SimpleFilter<Dsp::RBJ::HighPass, Channels>>();
            filter->setup(design.sample_rate_hz, design.frequency_hz, design.quality);
            return visit(*filter);
        }
        default:
            return STATUS_INVALID_ARGUMENT;
    }
}

// Runs interleaved f32 frames through a `Channels`-channel filter as the originals do: split into
// one float vector per channel, `process<float>`, interleave back.
template <int Channels>
int32_t process_f32(const oracle_filter_design& design, float* samples, size_t frames) {
    return with_filter<Channels>(design, [&](auto& filter) {
        std::array<std::vector<float>, Channels> planar;
        for (auto& channel : planar) {
            channel.resize(frames);
        }
        for (size_t frame = 0; frame < frames; ++frame) {
            for (int channel = 0; channel < Channels; ++channel) {
                planar[channel][frame] = samples[frame * Channels + channel];
            }
        }
        filter.template process<float>(static_cast<int>(frames), planar);
        for (size_t frame = 0; frame < frames; ++frame) {
            for (int channel = 0; channel < Channels; ++channel) {
                samples[frame * Channels + channel] = planar[channel][frame];
            }
        }
        return STATUS_OK;
    });
}

// Runs `body` and turns every C++ exception into STATUS_EXCEPTION.
template <typename Body>
int32_t guarded(Body body) {
    try {
        return body();
    } catch (...) {
        return STATUS_EXCEPTION;
    }
}

// A SimpleComp set up in the order clientData.hpp uses.
void set_up_compressor(chunkware_simple::SimpleComp& compressor,
                       const oracle_compressor_settings& settings) {
    compressor.setSampleRate(settings.sample_rate_hz);
    compressor.setThresh(settings.threshold_db);
    compressor.setRelease(settings.release_ms);
    compressor.setAttack(settings.attack_ms);
    compressor.setRatio(settings.ratio);
    compressor.initRuntime();
}

F3DAUDIO_VECTOR to_f3daudio(const oracle_vector& vector) {
    F3DAUDIO_VECTOR result;
    result.x = vector.x;
    result.y = vector.y;
    result.z = vector.z;
    return result;
}

}  // namespace

extern "C" {

// Filters `frames` interleaved frames of `channels` (1 or 2) f32 samples in place.
int32_t oracle_filter_process_f32(const oracle_filter_design* design, uint32_t channels,
                                  float* samples, size_t frames) {
    if (design == nullptr || (samples == nullptr && frames > 0)) {
        return STATUS_INVALID_ARGUMENT;
    }
    return guarded([&] {
        switch (channels) {
            case 1: return process_f32<1>(*design, samples, frames);
            case 2: return process_f32<2>(*design, samples, frames);
            default: return STATUS_INVALID_ARGUMENT;
        }
    });
}

// Writes the filter's linear gain |H| at each of `count` frequencies in hertz into `magnitudes`.
int32_t oracle_filter_magnitudes(const oracle_filter_design* design,
                                 const double* frequencies_hz, double* magnitudes, size_t count) {
    if (design == nullptr || ((frequencies_hz == nullptr || magnitudes == nullptr) && count > 0)) {
        return STATUS_INVALID_ARGUMENT;
    }
    return guarded([&] {
        return with_filter<1>(*design, [&](auto& filter) {
            for (size_t index = 0; index < count; ++index) {
                const double normalised = frequencies_hz[index] / design->sample_rate_hz;
                magnitudes[index] = std::abs(filter.response(normalised));
            }
            return STATUS_OK;
        });
    });
}

// Filters a stereo i16 block in place as helpers.hpp processFilterStereo does: sample / SHRT_MAX
// as float, process<float>, times gain, then SHRT_MAX above 1, SHRT_MIN below -1, and
// static_cast<short>(sample * (SHRT_MAX - 1)) otherwise.
int32_t oracle_filter_process_stereo_i16(const oracle_filter_design* design, int16_t* samples,
                                         size_t frames, float gain) {
    if (design == nullptr || (samples == nullptr && frames > 0)) {
        return STATUS_INVALID_ARGUMENT;
    }
    return guarded([&] {
        return with_filter<2>(*design, [&](auto& filter) {
            std::array<std::vector<float>, 2> planar;
            planar[0].resize(frames);
            planar[1].resize(frames);
            for (size_t frame = 0; frame < frames; ++frame) {
                for (int channel = 0; channel < 2; ++channel) {
                    planar[channel][frame] = static_cast<float>(samples[frame * 2 + channel]) /
                                             static_cast<float>(SHRT_MAX);
                }
            }
            filter.template process<float>(static_cast<int>(frames), planar);
            for (size_t frame = 0; frame < frames; ++frame) {
                for (int channel = 0; channel < 2; ++channel) {
                    const auto sample = planar[channel][frame] * gain;
                    short value;
                    if (sample > 1.0) {
                        value = SHRT_MAX;
                    } else if (sample < -1.0) {
                        value = SHRT_MIN;
                    } else {
                        value = static_cast<short>(sample * (SHRT_MAX - 1));
                    }
                    samples[frame * 2 + channel] = value;
                }
            }
            return STATUS_OK;
        });
    });
}

// Compresses `frames` interleaved stereo doubles in place with one SimpleComp; when `gains` is
// not null, a second SimpleComp with the same settings writes each frame's linear gain reduction
// into it, by running the stereo-linked key max(|l|, |r|) over unit samples.
int32_t oracle_simple_comp_process(const oracle_compressor_settings* settings, double* samples,
                                   size_t frames, double* gains) {
    if (settings == nullptr || (samples == nullptr && frames > 0) || settings->ratio <= 0.0) {
        return STATUS_INVALID_ARGUMENT;
    }
    return guarded([&] {
        chunkware_simple::SimpleComp compressor;
        chunkware_simple::SimpleComp gain_probe;
        set_up_compressor(compressor, *settings);
        set_up_compressor(gain_probe, *settings);
        for (size_t frame = 0; frame < frames; ++frame) {
            double& left = samples[frame * 2];
            double& right = samples[frame * 2 + 1];
            if (gains != nullptr) {
                double unit_left = 1.0;
                double unit_right = 1.0;
                const double key = std::fmax(std::fabs(left), std::fabs(right));
                gain_probe.process(unit_left, unit_right, key);
                gains[frame] = unit_left;
            }
            compressor.process(left, right);
        }
        return STATUS_OK;
    });
}

// Compresses a stereo i16 block in place as RadioEffect.hpp processCompressor does: each sample
// as static_cast<double>, SimpleComp::process, static_cast<short> back.
int32_t oracle_simple_comp_process_stereo_i16(const oracle_compressor_settings* settings,
                                              int16_t* samples, size_t frames) {
    if (settings == nullptr || (samples == nullptr && frames > 0) || settings->ratio <= 0.0) {
        return STATUS_INVALID_ARGUMENT;
    }
    return guarded([&] {
        chunkware_simple::SimpleComp compressor;
        set_up_compressor(compressor, *settings);
        for (size_t frame = 0; frame < frames; ++frame) {
            auto left = static_cast<double>(samples[frame * 2]);
            auto right = static_cast<double>(samples[frame * 2 + 1]);
            compressor.process(left, right);
            samples[frame * 2] = static_cast<short>(left);
            samples[frame * 2 + 1] = static_cast<short>(right);
        }
        return STATUS_OK;
    });
}

// Runs F3DAudioCalculate for a stereo output (SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT, as
// applyILD initialises it) and a one-channel emitter: the two matrix coefficients, their sum (the
// curve and cone volume, since the matrix splits it between the speakers), the emitter angle, the
// distance and the parameter checks that failed.
int32_t oracle_f3daudio_calculate(const oracle_spatial_scene* scene,
                                  oracle_spatial_result* result) {
    if (scene == nullptr || result == nullptr || scene->curve_point_count > MAX_CURVE_POINTS) {
        return STATUS_INVALID_ARGUMENT;
    }
    // 343.5 m/s is X3DAudio's X3DAUDIO_SPEED_OF_SOUND, which applyILD passes; F3DAudio.h has no
    // name for it, and the matrix does not depend on it.
    F3DAUDIO_HANDLE handle;
    F3DAudioInitialize(SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT, 343.5f, handle);

    F3DAUDIO_LISTENER listener{};
    listener.OrientFront = to_f3daudio(scene->listener_front);
    listener.OrientTop = to_f3daudio(scene->listener_top);
    listener.Position = to_f3daudio(scene->listener_position);
    listener.pCone = nullptr;

    F3DAUDIO_CONE cone{};
    cone.InnerAngle = scene->cone_inner_angle;
    cone.OuterAngle = scene->cone_outer_angle;
    cone.InnerVolume = scene->cone_inner_volume;
    cone.OuterVolume = scene->cone_outer_volume;

    std::array<F3DAUDIO_DISTANCE_CURVE_POINT, MAX_CURVE_POINTS> points{};
    for (uint32_t index = 0; index < scene->curve_point_count; ++index) {
        points[index].Distance = scene->curve_distances[index];
        points[index].DSPSetting = scene->curve_gains[index];
    }
    F3DAUDIO_DISTANCE_CURVE curve;
    curve.pPoints = points.data();
    curve.PointCount = scene->curve_point_count;

    F3DAUDIO_EMITTER emitter{};
    emitter.pCone = scene->has_cone != 0 ? &cone : nullptr;
    emitter.Position = to_f3daudio(scene->emitter_position);
    emitter.OrientFront = to_f3daudio(scene->emitter_front);
    emitter.OrientTop = to_f3daudio(scene->emitter_top);
    emitter.ChannelCount = 1;
    emitter.CurveDistanceScaler = scene->curve_distance_scaler;
    emitter.ChannelRadius = 1.0f;
    emitter.pVolumeCurve = scene->curve_point_count > 0 ? &curve : nullptr;

    std::array<float, 2> matrix{};
    F3DAUDIO_DSP_SETTINGS output{};
    output.SrcChannelCount = 1;
    output.DstChannelCount = 2;
    output.pMatrixCoefficients = matrix.data();

    assertion_failures = 0;
    F3DAudioCalculate(handle, &listener, &emitter,
                      F3DAUDIO_CALCULATE_MATRIX | F3DAUDIO_CALCULATE_EMITTER_ANGLE, &output);

    result->left = matrix[0];
    result->right = matrix[1];
    result->volume = matrix[0] + matrix[1];
    result->emitter_to_listener_angle = output.EmitterToListenerAngle;
    result->emitter_to_listener_distance = output.EmitterToListenerDistance;
    result->assertion_failures = assertion_failures;
    return STATUS_OK;
}

}  // extern "C"
