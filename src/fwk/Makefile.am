SUBDIRS = . base utils toolkit

CARGO_TARGET_DIR=@abs_top_builddir@/target

RUST_BIN_DIR=$(CARGO_TARGET_DIR)/bin
RUST_CXXBRIDGE=$(RUST_BIN_DIR)/cxxbridge

cxx_fwk_bindings.cpp: $(top_srcdir)/crates/npc-fwk/src/lib.rs
	@echo "Generating bindings $@..."
	@$(RUST_CXXBRIDGE) $< > $@

cxx_fwk_bindings.hpp: $(top_srcdir)/crates/npc-fwk/src/lib.rs
	@echo "Generating bindings header $@..."
	@$(RUST_CXXBRIDGE) --header $< > $@

BUILT_SOURCES = \
        cxx_fwk_bindings.cpp \
        cxx_fwk_bindings.hpp

CLEANFILES = $(BUILD_SOURCES)
