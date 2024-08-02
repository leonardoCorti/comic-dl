# comic-dl
this is a command line program to download comics online in cbz format to read them on e-readers

it can be also used directly on the kobo readers with KOreader installed with the enviroment variable `SSL_CERT_FILE=/mnt/onboard/.adds/koreader/data/ca.bunlde.crt`, for ease of use I reccomend writing a script like this
``` bash
#!/bin/sh
SSL_CERT_FILE=/mnt/onboard/.adds/koreader/data/ca.bunlde.crt ./comic-dl-armv7-linux <LINK_TO_COMIC>
```
it can be executed to downloade new issues and it will detect the already downloaded and avoid downloading them again.
# usage
`comic-dl LINK_TO_COMIC [-J\<number of threads\>]`

the first argument must be the link to the comic

it supports parallelization with the argument -J\<number of threads\>.

# websites supported
- zerocalcare.net
- readcomic.me
