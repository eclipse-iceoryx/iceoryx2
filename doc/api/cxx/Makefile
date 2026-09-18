.PHONY: docs clean clean-all setup help deps

# Configuration
DOXYGEN_AWESOME_VERSION = v2.5.0
DOXYGEN_AWESOME_URL = https://github.com/jothepro/doxygen-awesome-css.git
DOXYGEN_AWESOME_DIR = target/deps/doxygen-awesome

html: deps
	@echo "🔨 Building documentation..."
	@mkdir -p target
	@if [ ! -f Doxyfile.global ]; then doxygen -g Doxyfile.global; fi
	doxygen Doxyfile
	@echo "✅ Documentation built! Open: target/html/index.html"

deps:
	@mkdir -p target/deps
	@if [ ! -d "$(DOXYGEN_AWESOME_DIR)" ]; then \
		echo "⬇️  Downloading doxygen-awesome-css $(DOXYGEN_AWESOME_VERSION)..."; \
		git clone --depth 1 --branch $(DOXYGEN_AWESOME_VERSION) $(DOXYGEN_AWESOME_URL) $(DOXYGEN_AWESOME_DIR); \
		echo "✅ Dependencies downloaded!"; \
	else \
		echo "✅ Dependencies already present"; \
	fi

clean:
	rm -r --interactive=never target/html
	rm Doxyfile.global

clean-all:
	rm -r --interactive=never target
	rm Doxyfile.global

help:
	@echo "Available targets:"
	@echo "  html       - Build the documentation"
	@echo "  deps       - Download dependencies"
	@echo "  clean      - Remove only documentation, keep dependencies"
	@echo "  clean-all  - Remove all generated files including dependencies"
	@echo "  help       - Show this help message"
