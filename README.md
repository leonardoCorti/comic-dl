# comic-dl
this is a command line program to download comics online in cbz format to read them on e-readers

it can be executed to download new issues and it will detect the already downloaded and avoid downloading them again.

# usage
```bash
Usage: comic-dl.exe [OPTIONS] <COMIC_LINK>

Arguments:
  <COMIC_LINK>  The link to the comic

Options:
  -J, --threads <THREADS>        Number of threads to use for dowloading [default: 1]
  -S, --skip-start <SKIP_COUNT>  Number of issues to skip from the start [default: 0]
  -L, --skip-last <SKIP_COUNT>   Number of issues to skip from the last [default: 0]
  -p, --path <PATH>              Download path
      --pdf                      Download as PDF
      --kobo-install             Install to Kobo after download
  -I, --interactive              interactive mode (todo!)
  -h, --help                     Print help
  -V, --version                  Print version
```

with the -p flag a custom download path can be used, a subdirectory with the name of the comic will still be created so the same download path can be used with different comics and it will still be organized

# how to use on kobo e-reader

it can be also used directly on the kobo readers with KOreader installed, for ease of use I reccomend writing a script like this
``` bash
#!/bin/sh
cd "$(dirname "$0")"
./comic-dl-armv7-linux <LINK_TO_COMIC>
```
it can also be generated with the --kobo-install option, this way a directory called "install" will be created with the program and some scripts, they should be copied to the kobo all in the same directory and from koreader you can launch the program from the scirpts (file that end in .sh) with a long press

# websites supported
- zerocalcare.net
- readcomic.me
- scanita.org
