#ifndef _WIN64
#include <dirent.h>

int scandir_ext(const char *dir, struct dirent ***namelist) {
    return scandir(dir, namelist, 0, alphasort);
}
#endif
