
glibmm_deps = dependency('glibmm-2.68')
giomm_deps = dependency('giomm-2.68')
cairomm_deps = dependency('cairomm-1.16')
iptcdata_deps = dependency('libiptcdata')
lcms2_deps = dependency('lcms2')
lensfun_deps = dependency('lensfun')
# The needed for the C++ tests.
png16_deps = dependency('libpng16')
tiff_deps = dependency('libtiff-4')
jpeg_deps = dependency('libjpeg')
expat_deps = dependency('expat')
fftw3f = dependency('fftw3f')

rtdatadir = datadir / 'niepce' / 'rawtherapee'
install_data(
  'RawTherapee/rtengine/camconst.json',

  install_dir: rtdatadir
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
  'RawTherapee/rtengine/ipvibrance.cc',
  'RawTherapee/rtengine/ipwavelet.cc',
  'RawTherapee/rtengine/jdatasrc.cc',
  'RawTherapee/rtengine/jpeg_ijg/jpeg_memsrc.cc',
  'RawTherapee/rtengine/labimage.cc',
  'RawTherapee/rtengine/lcp.cc',
  'RawTherapee/rtengine/lmmse_demosaic.cc',
  'RawTherapee/rtengine/loadinitial.cc',
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

  'RawTherapee/rtexif/canonattribs.cc',
  'RawTherapee/rtexif/fujiattribs.cc',
  'RawTherapee/rtexif/kodakattribs.cc',
  'RawTherapee/rtexif/nikonattribs.cc',
  'RawTherapee/rtexif/olympusattribs.cc',
  'RawTherapee/rtexif/panasonicattribs.cc',
  'RawTherapee/rtexif/pentaxattribs.cc',
  'RawTherapee/rtexif/rtexif.cc',
  'RawTherapee/rtexif/sonyminoltaattribs.cc',
  'RawTherapee/rtexif/stdattribs.cc',

  'RawTherapee/rtgui/editcallbacks.cc',
  'RawTherapee/rtgui/multilangmgr.cc',
  'RawTherapee/rtgui/options.cc',
  'RawTherapee/rtgui/paramsedited.cc',
  'RawTherapee/rtgui/pathutils.cc',

  cpp_args: [
    '-DNPC_NOGUI=1',
    '-DMYFILE_MMAP=1',
    '-DGLIBMM_268=1',
    '-DUSE_STD_MUTEX=1'
  ],
  include_directories: [ 'RawTherapee' / 'rtengine' / 'klt' ],
  dependencies: [ glibmm_deps, giomm_deps, cairomm_deps, gtkmm_deps, iptcdata_deps, lcms2_deps, lensfun_deps ],
)
