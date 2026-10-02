# HRIR sphere credits

## IRC_1002_C.bin

- **What:** the head-related impulse responses of LISTEN subject 1002, compensated set (`C`):
  187 directions, 512 samples per ear at 44.1 kHz, triangulated into one sphere in the `.bin`
  format the `hrtf` crate reads.
- **Source:** mrDIMAS's hrir_sphere_builder,
  <https://github.com/mrDIMAS/hrir_sphere_builder/blob/e52a10ece678a2b80a0978f7cf23f3ad9cce41c3/hrtf_base/IRCAM/IRC_1002_C.bin>,
  at commit `e52a10ece678a2b80a0978f7cf23f3ad9cce41c3` (2021-11-17, the merge of "Right Handed
  coordinates"). The file itself was last rebuilt in commit
  `d53f469598736697a7904be8d48c30607b594ddc` ("Fix coordinate system to RH Cartesian"), so its
  points are right-handed with x right, y up and z backwards.
- **Downloaded:** 2026-10-03, unchanged byte for byte.
- **SHA-256:** `3ca6c20f616ccddb4d01b6862e311117276d3536d9d0c11541363518865866a2`
- **Size:** 772656 bytes; Git blob `868537b01cf80de6db25990b21efd49e08a55ae1`, the same blob
  GitHub lists for that path at that commit.

## The measurements

LISTEN HRTF database, IRCAM, http://recherche.ircam.fr/equipes/salles/listen/

The impulse responses were measured by IRCAM (Room Acoustics Team, Olivier Warusfel) and AKG in
the Listen project, a shared-cost RTD project of the European Commission's Information Society
Technologies programme (Fifth Framework Programme). The compensated set is diffuse-field
equalised by IRCAM: magnitude only, windowed to 512 points with the propagation delay removed.

## Terms of use

- **The LISTEN data:** IRCAM publishes the database for free download on the page above. That
  page and its download, context and bibliography pages state no licence text and no conditions;
  the credit above, naming IRCAM and the LISTEN database with its address, is the attribution
  given with every redistribution of this file, and the file is never edited.
- **The sphere file and its builder:** hrir_sphere_builder is under the MIT licence, Copyright (c)
  2019 Dmitry Stepanov, which covers the conversion of the IRCAM WAV files into this sphere.
- **The reader:** the `hrtf` crate 0.9.0 (MIT, Dmitry Stepanov) reads and resamples the file.
