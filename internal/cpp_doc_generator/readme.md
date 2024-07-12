1. install dependencies (archlinux):
```sh
yay -S python-exhale python-breathe python-sphinx python-sphinx_rtd_theme \
       python-sphinx-sitemap doxygen
```

2. generate documentation:
```sh
make html
```

3. open generated documentation in browser:
```
target/html/index.html
```
