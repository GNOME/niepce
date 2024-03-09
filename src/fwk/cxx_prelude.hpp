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
class SharedConfiguration;
typedef std::shared_ptr<SharedConfiguration> ConfigurationPtr;
class UndoHistory;
class UndoTransaction;
}

namespace ui {
class NiepceApplication;
}

namespace ncr {
class Image;
}
