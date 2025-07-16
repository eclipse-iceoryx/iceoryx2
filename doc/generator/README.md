# cpp_doc_generator

1. Install dependencies (archlinux):
   ```sh
   python -m venv .env
   source .env/bin/activate      # bash shell
   source .env/bin/activate.fish # fish shell
   pip install -r requirements.txt
   ```
2. Generate documentation:
   ```sh
   make html
   ```
3. Open generated documentation in browser:
   ```text
   target/html/index.html
   ```
