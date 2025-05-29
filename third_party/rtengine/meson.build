# Build RawTherapee for rtengine embedding.
#

openmp_deps = dependency('openmp')
cairomm_deps = dependency('cairomm-1.16')
iptcdata_deps = dependency('libiptcdata')
lcms2_deps = dependency('lcms2')
lensfun_deps = dependency('lensfun')
libraw_deps = dependency('libraw')
# The needed for the C++ tests.
png16_deps = dependency('libpng16')
tiff_deps = dependency('libtiff-4')
jpeg_deps = dependency('libjpeg')
expat_deps = dependency('expat')
fftw3f = dependency('fftw3f')
zlib_deps = dependency('zlib')

rtdatadir = datadir / 'niepce' / 'rawtherapee'
install_data(
  'RawTherapee/rtengine/camconst.json',

  install_dir: rtdatadir
)

dcpdatadir = rtdatadir / 'dcpprofiles'
install_data(
  'RawTherapee/rtdata/dcpprofiles/camera_model_aliases.json',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS 100D.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS 1300D.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS-1D Mark III.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS-1Ds Mark II.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS-1D X Mark III.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS 20D.dcp',
  'RawTherapee/rtdata/dcpprofiles/CANON EOS 250D.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS 400D.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS 40D.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS 450D.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS 50D.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS 550D.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS 5D.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS 5D Mark II.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS 5D Mark III.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS 5D Mark IV.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS 600D.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS 60D.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS 650D.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS 6D.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS 6D Mark II.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS 7D.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS 7D Mark II.dcp',
  'RawTherapee/rtdata/dcpprofiles/CANON EOS 800D.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS 90D.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS D60.dcp',
  'RawTherapee/rtdata/dcpprofiles/CANON EOS M50.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS M6 Mark II.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS R5.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS R6.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS R8.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS R.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon EOS RP.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon PowerShot G10.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon PowerShot G12.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon PowerShot G1 X Mark II.dcp',
  'RawTherapee/rtdata/dcpprofiles/CANON POWERSHOT G5 X MARK II.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon PowerShot G7 X.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon PowerShot S110.dcp',
  'RawTherapee/rtdata/dcpprofiles/Canon PowerShot S120.dcp',
  'RawTherapee/rtdata/dcpprofiles/FUJIFILM DBP for GX680.dcp',
  'RawTherapee/rtdata/dcpprofiles/FUJIFILM FinePix F600EXR.dcp',
  'RawTherapee/rtdata/dcpprofiles/Fujifilm FinePix S9500.dcp',
  'RawTherapee/rtdata/dcpprofiles/FUJIFILM GFX 50R.dcp',
  'RawTherapee/rtdata/dcpprofiles/FUJIFILM X100S.dcp',
  'RawTherapee/rtdata/dcpprofiles/FUJIFILM X100T.dcp',
  'RawTherapee/rtdata/dcpprofiles/FUJIFILM X-A5.dcp',
  'RawTherapee/rtdata/dcpprofiles/FUJIFILM X-A7.dcp',
  'RawTherapee/rtdata/dcpprofiles/Fujifilm X-E1.dcp',
  'RawTherapee/rtdata/dcpprofiles/Fujifilm X-E2.dcp',
  'RawTherapee/rtdata/dcpprofiles/FUJIFILM X-E3.dcp',
  'RawTherapee/rtdata/dcpprofiles/FUJIFILM X-H1.dcp',
  'RawTherapee/rtdata/dcpprofiles/FUJIFILM X-Pro2.dcp',
  'RawTherapee/rtdata/dcpprofiles/FUJIFILM X-Pro3.dcp',
  'RawTherapee/rtdata/dcpprofiles/FUJIFILM X-S1.dcp',
  'RawTherapee/rtdata/dcpprofiles/FUJIFILM X-S10.dcp',
  'RawTherapee/rtdata/dcpprofiles/FUJIFILM X-T10.dcp',
  'RawTherapee/rtdata/dcpprofiles/FUJIFILM X-T1.dcp',
  'RawTherapee/rtdata/dcpprofiles/FUJIFILM X-T20.dcp',
  'RawTherapee/rtdata/dcpprofiles/FUJIFILM X-T2.dcp',
  'RawTherapee/rtdata/dcpprofiles/FUJIFILM X-T30.dcp',
  'RawTherapee/rtdata/dcpprofiles/FUJIFILM X-T3.dcp',
  'RawTherapee/rtdata/dcpprofiles/FUJIFILM X-T4.dcp',
  'RawTherapee/rtdata/dcpprofiles/Leaf Aptus 75.dcp',
  'RawTherapee/rtdata/dcpprofiles/LG Mobile LG-H815.dcp',
  'RawTherapee/rtdata/dcpprofiles/MINOLTA DYNAX 7D.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON COOLPIX P7800.dcp',
  'RawTherapee/rtdata/dcpprofiles/Nikon D200.dcp',
  'RawTherapee/rtdata/dcpprofiles/Nikon D3000.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON D300.dcp',
  'RawTherapee/rtdata/dcpprofiles/Nikon D3100.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON D3300.dcp',
  'RawTherapee/rtdata/dcpprofiles/Nikon D3S.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON D5000.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON D500.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON D50.dcp',
  'RawTherapee/rtdata/dcpprofiles/Nikon D5100.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON D5300.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON D5600.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON D600.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON D610.dcp',
  'RawTherapee/rtdata/dcpprofiles/Nikon D7000.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON D700.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON D70s.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON D7100.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON D7200.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON D7500.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON D750.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON D800.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON D800E.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON D80.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON D810.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON D850.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON Z 50.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON Z 5.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON Z 6.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON Z 6_2.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON Z 7.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON Z 8.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON Z 9.dcp',
  'RawTherapee/rtdata/dcpprofiles/NIKON Z F.dcp',
  'RawTherapee/rtdata/dcpprofiles/Olympus E-1.dcp',
  'RawTherapee/rtdata/dcpprofiles/OLYMPUS E-510.dcp',
  'RawTherapee/rtdata/dcpprofiles/Olympus E-520.dcp',
  'RawTherapee/rtdata/dcpprofiles/OLYMPUS E-M1.dcp',
  'RawTherapee/rtdata/dcpprofiles/OLYMPUS E-M10.dcp',
  'RawTherapee/rtdata/dcpprofiles/OLYMPUS E-M1MarkII.dcp',
  'RawTherapee/rtdata/dcpprofiles/Olympus E-M5.dcp',
  'RawTherapee/rtdata/dcpprofiles/OLYMPUS E-M5MarkII.dcp',
  'RawTherapee/rtdata/dcpprofiles/Olympus E-P2.dcp',
  'RawTherapee/rtdata/dcpprofiles/Olympus XZ-1.dcp',
  'RawTherapee/rtdata/dcpprofiles/Panasonic DC-G9.dcp',
  'RawTherapee/rtdata/dcpprofiles/Panasonic DC-GX9.dcp',
  'RawTherapee/rtdata/dcpprofiles/Panasonic DC-S1.dcp',
  'RawTherapee/rtdata/dcpprofiles/Panasonic DC-S5.dcp',
  'RawTherapee/rtdata/dcpprofiles/PANASONIC DC-S5M2.dcp',
  'RawTherapee/rtdata/dcpprofiles/Panasonic DC-TZ91.dcp',
  'RawTherapee/rtdata/dcpprofiles/Panasonic DMC-FZ1000.dcp',
  'RawTherapee/rtdata/dcpprofiles/Panasonic DMC-FZ150.dcp',
  'RawTherapee/rtdata/dcpprofiles/Panasonic DMC-FZ35.dcp',
  'RawTherapee/rtdata/dcpprofiles/Panasonic DMC-FZ38.dcp',
  'RawTherapee/rtdata/dcpprofiles/Panasonic DMC-G1.dcp',
  'RawTherapee/rtdata/dcpprofiles/Panasonic DMC-G3.dcp',
  'RawTherapee/rtdata/dcpprofiles/Panasonic DMC-G5.dcp',
  'RawTherapee/rtdata/dcpprofiles/Panasonic DMC-GH1.dcp',
  'RawTherapee/rtdata/dcpprofiles/Panasonic DMC-GH2.dcp',
  'RawTherapee/rtdata/dcpprofiles/Panasonic DMC-GX7.dcp',
  'RawTherapee/rtdata/dcpprofiles/Panasonic DMC-GX85.dcp',
  'RawTherapee/rtdata/dcpprofiles/PANASONIC DMC-LX100.dcp',
  'RawTherapee/rtdata/dcpprofiles/PENTAX K10D.dcp',
  'RawTherapee/rtdata/dcpprofiles/Pentax K200D.dcp',
  'RawTherapee/rtdata/dcpprofiles/PENTAX K-50.dcp',
  'RawTherapee/rtdata/dcpprofiles/PENTAX K-5.dcp',
  'RawTherapee/rtdata/dcpprofiles/PENTAX K-5 II.dcp',
  'RawTherapee/rtdata/dcpprofiles/Pentax K-r.dcp',
  'RawTherapee/rtdata/dcpprofiles/RICOH GR III.dcp',
  'RawTherapee/rtdata/dcpprofiles/RICOH PENTAX K-1.dcp',
  'RawTherapee/rtdata/dcpprofiles/RICOH PENTAX K-3.dcp',
  'RawTherapee/rtdata/dcpprofiles/samsung SM-G930V.dcp',
  'RawTherapee/rtdata/dcpprofiles/SONY DSC-RX100M6.dcp',
  'RawTherapee/rtdata/dcpprofiles/SONY DSLR-A580.dcp',
  'RawTherapee/rtdata/dcpprofiles/Sony DSLR-A700.dcp',
  'RawTherapee/rtdata/dcpprofiles/Sony DSLR-A900.dcp',
  'RawTherapee/rtdata/dcpprofiles/SONY ILCE-6000.dcp',
  'RawTherapee/rtdata/dcpprofiles/SONY ILCE-6300.dcp',
  'RawTherapee/rtdata/dcpprofiles/SONY ILCE-6400.dcp',
  'RawTherapee/rtdata/dcpprofiles/SONY ILCE-6500.dcp',
  'RawTherapee/rtdata/dcpprofiles/SONY ILCE-6600.dcp',
  'RawTherapee/rtdata/dcpprofiles/SONY ILCE-7.dcp',
  'RawTherapee/rtdata/dcpprofiles/SONY ILCE-7C.dcp',
  'RawTherapee/rtdata/dcpprofiles/SONY ILCE-7M2.dcp',
  'RawTherapee/rtdata/dcpprofiles/SONY ILCE-7M3.dcp',
  'RawTherapee/rtdata/dcpprofiles/SONY ILCE-7M4.dcp',
  'RawTherapee/rtdata/dcpprofiles/SONY ILCE-7RM3.dcp',
  'RawTherapee/rtdata/dcpprofiles/SONY ILCE-7RM4.dcp',
  'RawTherapee/rtdata/dcpprofiles/SONY ILCE-7SM3.dcp',
  'RawTherapee/rtdata/dcpprofiles/SONY ILCE-9.dcp',
  'RawTherapee/rtdata/dcpprofiles/Sony NEX-5N.dcp',
  'RawTherapee/rtdata/dcpprofiles/Sony SLT-A55V.dcp',
  'RawTherapee/rtdata/dcpprofiles/SONY SLT-A99V.dcp',
  'RawTherapee/rtdata/dcpprofiles/YI TECHNOLOGY M1.dcp',

  install_dir: dcpdatadir
)

profilesdatadir = rtdatadir / 'profiles'
install_data(
  'RawTherapee/rtdata/profiles/Auto-Matched Curve - ISO High.pp3',
  'RawTherapee/rtdata/profiles/Auto-Matched Curve - ISO Low.pp3',
  'RawTherapee/rtdata/profiles/Auto-Matched Curve - ISO Medium.pp3',
  'RawTherapee/rtdata/profiles/Film Negative - Black and White.pp3',
  'RawTherapee/rtdata/profiles/Film Negative.pp3',
  'RawTherapee/rtdata/profiles/Standard Film Curve - ISO High.pp3',
  'RawTherapee/rtdata/profiles/Standard Film Curve - ISO Low.pp3',
  'RawTherapee/rtdata/profiles/Standard Film Curve - ISO Medium.pp3',
  'RawTherapee/rtdata/profiles/Unclipped.pp3',

  install_dir: profilesdatadir
)

profiles2datadir = profilesdatadir / 'Pop'
install_data(
  'RawTherapee/rtdata/profiles/Pop/Pop 1.pp3',
  'RawTherapee/rtdata/profiles/Pop/Pop 2 Lab.pp3',
  'RawTherapee/rtdata/profiles/Pop/Pop 3 Skin.pp3',
  'RawTherapee/rtdata/profiles/Pop/Pop 4 Black-and-White.pp3',

  install_dir: profiles2datadir
)

profiles3datadir = profilesdatadir / 'Pixel Shift'
install_data(
  'RawTherapee/rtdata/profiles/Pixel Shift/PS ISO High.pp3',
  'RawTherapee/rtdata/profiles/Pixel Shift/PS ISO Low.pp3',
  'RawTherapee/rtdata/profiles/Pixel Shift/PS ISO Medium.pp3',
  'RawTherapee/rtdata/profiles/Pixel Shift/PS No Motion.pp3',

  install_dir: profiles3datadir
)

profiles4datadir = profilesdatadir / 'Non-raw'
install_data(
  'RawTherapee/rtdata/profiles/Non-raw/Brighten.pp3',

  install_dir: profiles4datadir
)

iccdatadir = rtdatadir / 'iccprofiles'
iccinputdatadir = iccdatadir / 'input'
iccoutputdatadir = iccdatadir / 'output'

install_data(
  'RawTherapee/rtdata/iccprofiles/input/Canon EOS-1D Mark III.icc',
  'RawTherapee/rtdata/iccprofiles/input/Canon EOS 20D.icc',
  'RawTherapee/rtdata/iccprofiles/input/Canon EOS 40D.icc',
  'RawTherapee/rtdata/iccprofiles/input/Canon EOS 450D.icc',
  'RawTherapee/rtdata/iccprofiles/input/Canon EOS 550D.icc',
  'RawTherapee/rtdata/iccprofiles/input/Canon EOS 5D.icc',
  'RawTherapee/rtdata/iccprofiles/input/Canon PowerShot G10.icc',
  'RawTherapee/rtdata/iccprofiles/input/Canon PowerShot G12.icc',
  'RawTherapee/rtdata/iccprofiles/input/Nikon D200.icc',
  'RawTherapee/rtdata/iccprofiles/input/Nikon D3000.icc',
  'RawTherapee/rtdata/iccprofiles/input/Nikon D3100.icc',
  'RawTherapee/rtdata/iccprofiles/input/Nikon D3S.icc',
  'RawTherapee/rtdata/iccprofiles/input/Nikon D7000.icc',
  'RawTherapee/rtdata/iccprofiles/input/Nikon D700.icc',
  'RawTherapee/rtdata/iccprofiles/input/Olympus E-P2.icc',
  'RawTherapee/rtdata/iccprofiles/input/Panasonic DMC-FZ150.icc',
  'RawTherapee/rtdata/iccprofiles/input/Panasonic DMC-FZ35.icc',
  'RawTherapee/rtdata/iccprofiles/input/Panasonic DMC-FZ38.icc',
  'RawTherapee/rtdata/iccprofiles/input/Panasonic DMC-G1.icc',
  'RawTherapee/rtdata/iccprofiles/input/Panasonic DMC-G3.icc',
  'RawTherapee/rtdata/iccprofiles/input/Panasonic DMC-GH1.icc',
  'RawTherapee/rtdata/iccprofiles/input/Panasonic DMC-GH2.icc',
  'RawTherapee/rtdata/iccprofiles/input/Pentax K200D.icc',
  'RawTherapee/rtdata/iccprofiles/input/sd14-bl15-crop-matrix-gamma-wp10.icm',
  'RawTherapee/rtdata/iccprofiles/input/sd14-bl15-crop-matrix-gamma-wp11.icm',
  'RawTherapee/rtdata/iccprofiles/input/sd14-bl15-crop-matrix-gamma-wp12.icm',
  'RawTherapee/rtdata/iccprofiles/input/sd1_merrill_cloudy8140-CROP-WP10.icm',
  'RawTherapee/rtdata/iccprofiles/input/sd1_merrill_cloudy8140-CROP-WP11.icm',
  'RawTherapee/rtdata/iccprofiles/input/sd1_merrill_sunny8161-crop-wp10.icm',
  'RawTherapee/rtdata/iccprofiles/input/sd1_merrill_sunny8161-crop-wp11.icm',
  'RawTherapee/rtdata/iccprofiles/input/sd1_merrill_tungsten8130-CROP-WP10.icm',
  'RawTherapee/rtdata/iccprofiles/input/sd1_merrill_tungsten8130-CROP-WP11.icm',
  'RawTherapee/rtdata/iccprofiles/input/Sony DSLR-A700.icc',
  'RawTherapee/rtdata/iccprofiles/input/Sony DSLR-A900.icc',
  'RawTherapee/rtdata/iccprofiles/input/Sony SLT-A55V.icc',

  install_dir: iccinputdatadir
)

install_data(
  'RawTherapee/rtdata/iccprofiles/output/DCI-P3 D65.icc',
  'RawTherapee/rtdata/iccprofiles/output/DCI-P3 Theater.icc',
  'RawTherapee/rtdata/iccprofiles/output/RTv2_ACES-AP0.icc',
  'RawTherapee/rtdata/iccprofiles/output/RTv2_ACES-AP1.icc',
  'RawTherapee/rtdata/iccprofiles/output/RTv2_Best.icc',
  'RawTherapee/rtdata/iccprofiles/output/RTv2_Beta.icc',
  'RawTherapee/rtdata/iccprofiles/output/RTv2_Bruce.icc',
  'RawTherapee/rtdata/iccprofiles/output/RTv2_Large.icc',
  'RawTherapee/rtdata/iccprofiles/output/RTv2_Medium.icc',
  'RawTherapee/rtdata/iccprofiles/output/RTv2_Rec2020.icc',
  'RawTherapee/rtdata/iccprofiles/output/RTv2_sRGB.icc',
  'RawTherapee/rtdata/iccprofiles/output/RTv2_Wide.icc',
  'RawTherapee/rtdata/iccprofiles/output/RTv4_ACES-AP0.icc',
  'RawTherapee/rtdata/iccprofiles/output/RTv4_ACES-AP1.icc',
  'RawTherapee/rtdata/iccprofiles/output/RTv4_Best.icc',
  'RawTherapee/rtdata/iccprofiles/output/RTv4_Beta.icc',
  'RawTherapee/rtdata/iccprofiles/output/RTv4_Bruce.icc',
  'RawTherapee/rtdata/iccprofiles/output/RTv4_Large.icc',
  'RawTherapee/rtdata/iccprofiles/output/RTv4_Medium.icc',
  'RawTherapee/rtdata/iccprofiles/output/RTv4_Rec2020.icc',
  'RawTherapee/rtdata/iccprofiles/output/RTv4_sRGB.icc',
  'RawTherapee/rtdata/iccprofiles/output/RTv4_Wide.icc',

  install_dir: iccoutputdatadir
)


rtengine_version_data = configuration_data()
rtengine_version_data.set('RTVERSION', '"niepce"')
rtengine_version_data.set('CACHE_NAME_SUFFIX', '"npc"')

configure_file(
  configuration: rtengine_version_data,
  input: 'version.h.in',
  output: 'version.h'
)

rtengine_cfg_data = configuration_data()
rtengine_cfg_data.set_quoted('DATA_SEARCH_PATH', rtdatadir)

# This will use the dependency installed lensfun database.
# It might be out of date.
lensfundir = lensfun_deps.get_variable('datadir') / 'lensfun' / 'version_1'
rtengine_cfg_data.set_quoted('LENSFUN_DB_PATH', lensfundir)

configure_file(
  configuration: rtengine_cfg_data,
  input: 'npc_rtconfig.h.in',
  output: 'npc_rtconfig.h'
)

rtengine = static_library(
  'rtengine',

  'RawTherapee/rtengine/alpha.cc',
  'RawTherapee/rtengine/ahd_demosaic_RT.cc',
  'RawTherapee/rtengine/amaze_demosaic_RT.cc',
  'RawTherapee/rtengine/badpixels.cc',
  'RawTherapee/rtengine/bayer_bilinear_demosaic.cc',
  'RawTherapee/rtengine/boxblur.cc',
  'RawTherapee/rtengine/canon_cr3_decoder.cc',
  'RawTherapee/rtengine/CA_correct_RT.cc',
  'RawTherapee/rtengine/calc_distort.cc',
  'RawTherapee/rtengine/camconst.cc',
  'RawTherapee/rtengine/capturesharpening.cc',
  'RawTherapee/rtengine/cfa_linedn_RT.cc',
  'RawTherapee/rtengine/ciecam02.cc',
  'RawTherapee/rtengine/cieimage.cc',
  'RawTherapee/rtengine/cJSON.c',
  'RawTherapee/rtengine/clutstore.cc',
  'RawTherapee/rtengine/color.cc',
  'RawTherapee/rtengine/colortemp.cc',
  'RawTherapee/rtengine/coord.cc',
  'RawTherapee/rtengine/cplx_wavelet_dec.cc',
  'RawTherapee/rtengine/curves.cc',
  'RawTherapee/rtengine/dcp.cc',
  'RawTherapee/rtengine/dcraw.cc',
  'RawTherapee/rtengine/dcrop.cc',
  'RawTherapee/rtengine/demosaic_algos.cc',
  'RawTherapee/rtengine/dfmanager.cc',
  'RawTherapee/rtengine/diagonalcurves.cc',
  'RawTherapee/rtengine/dirpyr_equalizer.cc',
  'RawTherapee/rtengine/dual_demosaic_RT.cc',
  'RawTherapee/rtengine/dynamicprofile.cc',
  'RawTherapee/rtengine/eahd_demosaic.cc',
  'RawTherapee/rtengine/EdgePreservingDecomposition.cc',
  'RawTherapee/rtengine/fast_demo.cc',
  'RawTherapee/rtengine/ffmanager.cc',
  'RawTherapee/rtengine/filmnegativeproc.cc',
  'RawTherapee/rtengine/flatcurves.cc',
  'RawTherapee/rtengine/FTblockDN.cc',
  'RawTherapee/rtengine/gamutwarning.cc',
  'RawTherapee/rtengine/gauss.cc',
  'RawTherapee/rtengine/green_equil_RT.cc',
  'RawTherapee/rtengine/guidedfilter.cc',
  'RawTherapee/rtengine/hilite_recon.cc',
  'RawTherapee/rtengine/histmatching.cc',
  'RawTherapee/rtengine/hphd_demosaic_RT.cc',
  'RawTherapee/rtengine/iccjpeg.cc',
  'RawTherapee/rtengine/iccstore.cc',
  'RawTherapee/rtengine/iimage.cc',
  'RawTherapee/rtengine/image16.cc',
  'RawTherapee/rtengine/image8.cc',
  'RawTherapee/rtengine/imagedata.cc',
  'RawTherapee/rtengine/imagedimensions.cc',
  'RawTherapee/rtengine/imagefloat.cc',
  'RawTherapee/rtengine/imageio.cc',
  'RawTherapee/rtengine/imagesource.cc',
  'RawTherapee/rtengine/improccoordinator.cc',
  'RawTherapee/rtengine/improcfun.cc',
  'RawTherapee/rtengine/impulse_denoise.cc',
  'RawTherapee/rtengine/init.cc',
  'RawTherapee/rtengine/ipdehaze.cc',
  'RawTherapee/rtengine/ipgrain.cc',
  'RawTherapee/rtengine/iplab2rgb.cc',
  'RawTherapee/rtengine/iplocallab.cc',
  'RawTherapee/rtengine/iplabregions.cc',
  'RawTherapee/rtengine/iplocalcontrast.cc',
  'RawTherapee/rtengine/ipresize.cc',
  'RawTherapee/rtengine/ipretinex.cc',
  'RawTherapee/rtengine/ipshadowshighlights.cc',
  'RawTherapee/rtengine/ipsharpen.cc',
  'RawTherapee/rtengine/ipsharpenedges.cc',
  'RawTherapee/rtengine/ipsoftlight.cc',
  'RawTherapee/rtengine/iptransform.cc',
  'RawTherapee/rtengine/iptoneequalizer.cc',
  'RawTherapee/rtengine/ipvibrance.cc',
  'RawTherapee/rtengine/ipwavelet.cc',
  'RawTherapee/rtengine/jdatasrc.cc',
  'RawTherapee/rtengine/jpeg_ijg/jpeg_memsrc.cc',
  'RawTherapee/rtengine/labimage.cc',
  'RawTherapee/rtengine/lcp.cc',
  'RawTherapee/rtengine/lensmetadata.cc',
  'RawTherapee/rtengine/lmmse_demosaic.cc',
  'RawTherapee/rtengine/loadinitial.cc',
  'RawTherapee/rtengine/metadata.cc',
  'RawTherapee/rtengine/munselllch.cc',
  'RawTherapee/rtengine/myfile.cc',
  'RawTherapee/rtengine/panasonic_decoders.cc',
  'RawTherapee/rtengine/pdaflinesfilter.cc',
  'RawTherapee/rtengine/perspectivecorrection.cc',
  'RawTherapee/rtengine/PF_correct_RT.cc',
  'RawTherapee/rtengine/pipettebuffer.cc',
  'RawTherapee/rtengine/pixelshift.cc',
  'RawTherapee/rtengine/previewimage.cc',
  'RawTherapee/rtengine/processingjob.cc',
  'RawTherapee/rtengine/procparams.cc',
  'RawTherapee/rtengine/profilestore.cc',
  'RawTherapee/rtengine/rawflatfield.cc',
  'RawTherapee/rtengine/rawimage.cc',
  'RawTherapee/rtengine/rawimagesource.cc',
  'RawTherapee/rtengine/rcd_demosaic.cc',
  'RawTherapee/rtengine/refreshmap.cc',
  'RawTherapee/rtengine/rt_algo.cc',
  'RawTherapee/rtengine/rtlensfun.cc',
  'RawTherapee/rtengine/rtthumbnail.cc',
  'RawTherapee/rtengine/shmap.cc',
  'RawTherapee/rtengine/simpleprocess.cc',
  'RawTherapee/rtengine/spot.cc',
  'RawTherapee/rtengine/stdimagesource.cc',
  'RawTherapee/rtengine/tmo_fattal02.cc',
  'RawTherapee/rtengine/utils.cc',
  'RawTherapee/rtengine/vng4_demosaic_RT.cc',
  'RawTherapee/rtengine/xtrans_demosaic.cc',

  'RawTherapee/rtengine/klt/convolve.cc',
  'RawTherapee/rtengine/klt/error.cc',
  'RawTherapee/rtengine/klt/klt.cc',
  'RawTherapee/rtengine/klt/klt_util.cc',
  'RawTherapee/rtengine/klt/pnmio.cc',
  'RawTherapee/rtengine/klt/pyramid.cc',
  'RawTherapee/rtengine/klt/selectGoodFeatures.cc',
  'RawTherapee/rtengine/klt/storeFeatures.cc',
  'RawTherapee/rtengine/klt/trackFeatures.cc',
  'RawTherapee/rtengine/klt/writeFeatures.cc',

  'RawTherapee/rtgui/editcallbacks.cc',
  'RawTherapee/rtgui/multilangmgr.cc',
  'RawTherapee/rtgui/options.cc',
  'RawTherapee/rtgui/paramsedited.cc',
  'RawTherapee/rtgui/pathutils.cc',

  cpp_args: [
    '-DNPC_NOGUI=1',
    '-DMYFILE_MMAP=1',
    '-DGLIBMM_268=1',
    '-DUSE_STD_MUTEX=1',
    '-ftree-vectorize',
    # warnings for things we don't control in that tree.
    '-Wno-deprecated-declarations'
  ],
  include_directories: [
    'RawTherapee' / 'rtengine' / 'klt',
    'RawTherapee'
  ],
  dependencies: [ glibmm_deps, giomm_deps, cairomm_deps, iptcdata_deps, lcms2_deps, lensfun_deps, libraw_deps, openmp_deps, exiv2_deps ],
)
