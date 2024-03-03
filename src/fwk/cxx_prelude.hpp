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
class WrappedPropertyBag;
class SharedConfiguration;
typedef std::shared_ptr<SharedConfiguration> ConfigurationPtr;
class UndoHistory;
class UndoTransaction;
}

namespace eng {
class Label;
}

namespace ui {
class EditLabels;
class NiepceApplication;
}

namespace mapm {
class MapModule;
}

namespace ncr {
class Image;
}
