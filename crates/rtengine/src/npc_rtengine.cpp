
#include <glibmm/ustring.h>
#include <giomm/init.h>
#include "npc_rtconfig.h"
#include "npc_rtengine.h"

extern Glib::ustring argv0;

namespace rtengine {
  void init_() {
    argv0 = DATA_SEARCH_PATH;
    Gio::init();
  }
}
