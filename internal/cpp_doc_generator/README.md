# cpp_doc_generator

1. Install dependencies (archlinux):
   ```sh
   yay -S python-exhale python-breathe python-sphinx python-sphinx_rtd_theme \
          python-sphinx-sitemap doxygen
   ```
2. Generate documentation:
   ```sh
   make html
   ```
3. Open generated documentation in browser:
   ```text
   target/html/index.html
   ```
