# comic-dl
this is a command line program to download comics online in cbz format to read them on e-readers

it can be also used directly on the kobo readers with KOreader installed with the enviroment variable `SSL_CERT_FILE=/mnt/onboard/.adds/koreader/data/ca.bunlde.crt`, for ease of use I reccomend writing a script like this
``` bash
#!/bin/sh
cd /mnt/onboard/<directory with the program
./comic-dl-armv7-linux <LINK_TO_COMIC>
```
it can be executed to downloade new issues and it will detect the already downloaded and avoid downloading them again.
# usage
`comic-dl [-J<number of threads>] [-p <download path>] [link to the comic]`

it supports parallelization with the argument -J\<number of threads\>.

with the -p flag a custom download path can be used, a subdirectory with the name of the comic will still be created so the same download path can be used with different comics and it will still be organized

if launched without arguments it will ask for the link from terminal

# websites supported
- zerocalcare.net
- readcomic.me
- scanita.org
