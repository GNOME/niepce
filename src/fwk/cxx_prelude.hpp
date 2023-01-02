/*
 * niepce - fwk/cxx_prelude.hpp
 */

#pragma once

#include <memory>

// things that need to be declared before anything.
// early "extern C++"
// And that the implementation needs too.
namespace fwk {
class Application;
class PropertyValue;
class WrappedPropertyBag;
class SharedConfiguration;
typedef std::shared_ptr<SharedConfiguration> ConfigurationPtr;
class UndoHistory;
class UndoTransaction;
}

namespace eng {
class Label;
class LcChannel;
class LibFile;
class LibNotification;
class ThumbnailCache;
class IImporter;
}

namespace ui {
class ModuleShell;
class GridViewModule;
class EditLabels;
class MetaDataPaneController;
}

namespace mapm {
class MapModule;
}

namespace dr {
class DarkroomModule;
}

typedef struct _GdkPixbuf GdkPixbuf;
typedef struct _GtkWidget GtkWidget;

namespace npc {
class LnListener;
}
