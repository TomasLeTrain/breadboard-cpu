#include <assert.h>
#include <inttypes.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

void binary_file_to_logisim() {
  unsigned char buf[16] = {0};
  size_t bytes = 0, bytes_read, i, readsz = sizeof buf;
  FILE *fp = fopen("charset.out", "rb");

  if (!fp) {
    fprintf(stderr, "failed!\n");
    return;
  }
  printf("v3.0 hex words plain\n");

  while ((bytes = fread(buf, sizeof *buf, readsz, fp)) == readsz) {
    for (i = 0; i < 16; i++) {
      printf("%02X", buf[i]);
      if (bytes_read % 16 == 15) {
        printf("\n");
      } else {
        printf(" ");
      }
      bytes_read++;
    }
  }
  for (i = 0; i < bytes; i++) {
    printf("%02X", buf[i]);
    if (bytes_read % 16 == 15) {
      printf("\n");
    } else {
      printf(" ");
    }
    bytes_read++;
  }
	printf("total bytes: %d\n", bytes_read);

  fclose(fp);
}

int main() {
  binary_file_to_logisim();
}
