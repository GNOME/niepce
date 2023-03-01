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
class LibFile;
class LibNotification;
}

namespace ui {
class ModuleShell;
class GridViewModule;
class EditLabels;
class MetaDataPaneController;
class NiepceApplication;
}

namespace mapm {
class MapModule;
}

namespace dr {
class DarkroomModule;
}

namespace npc {
class LnListener;
}
