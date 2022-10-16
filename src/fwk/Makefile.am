SUBDIRS = . base utils toolkit

CARGO_TARGET_DIR=@abs_top_builddir@/target

RUST_BIN_DIR=$(CARGO_TARGET_DIR)/bin
RUST_CXXBRIDGE=$(RUST_BIN_DIR)/cxxbridge

cxx_colour_bindings.cpp: $(top_srcdir)/crates/npc-fwk/src/base/rgbcolour.rs
	@echo "Generating bindings $@..."
	@$(RUST_CXXBRIDGE) $< > $@

cxx_colour_bindings.hpp: $(top_srcdir)/crates/npc-fwk/src/base/rgbcolour.rs
	@echo "Generating bindings header $@..."
	@$(RUST_CXXBRIDGE) --header $< > $@

cxx_fwk_bindings.cpp: $(top_srcdir)/crates/npc-fwk/src/lib.rs
	@echo "Generating bindings $@..."
	@$(RUST_CXXBRIDGE) $< > $@

cxx_fwk_bindings.hpp: $(top_srcdir)/crates/npc-fwk/src/lib.rs
	@echo "Generating bindings header $@..."
	@$(RUST_CXXBRIDGE) --header $< > $@

cxx_widgets_bindings.cpp: $(top_srcdir)/crates/npc-fwk/src/toolkit/widgets/metadata_widget.rs
	@echo "Generating bindings $@..."
	@$(RUST_CXXBRIDGE) $< > $@

cxx_widgets_bindings.hpp: $(top_srcdir)/crates/npc-fwk/src/toolkit/widgets/metadata_widget.rs
	@echo "Generating bindings header $@..."
	@$(RUST_CXXBRIDGE) --header $< > $@

cxx_eng_bindings.cpp: $(top_srcdir)/crates/npc-engine/src/lib.rs
	@echo "Generating bindings $@..."
	@$(RUST_CXXBRIDGE) $< > $@

cxx_eng_bindings.hpp: $(top_srcdir)/crates/npc-engine/src/lib.rs
	@echo "Generating bindings header $@..."
	@$(RUST_CXXBRIDGE) --header $< > $@

cxx_npc_bindings.cpp: $(top_srcdir)/niepce-main/src/lib.rs
	@echo "Generating bindings $@..."
	@$(RUST_CXXBRIDGE) $< > $@

cxx_npc_bindings.hpp: $(top_srcdir)/niepce-main/src/lib.rs
	@echo "Generating bindings header $@..."
	@$(RUST_CXXBRIDGE) --header $< > $@

BUILT_SOURCES = \
        cxx_colour_bindings.cpp \
        cxx_colour_bindings.hpp \
        cxx_fwk_bindings.cpp \
        cxx_fwk_bindings.hpp \
        cxx_eng_bindings.cpp \
        cxx_eng_bindings.hpp \
        cxx_widgets_bindings.cpp \
        cxx_widgets_bindings.hpp \
        cxx_npc_bindings.cpp \
        cxx_npc_bindings.hpp \
        $(NULL)

CLEANFILES = $(BUILD_SOURCES)
