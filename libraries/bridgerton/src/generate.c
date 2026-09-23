// Target metadata runner, linked with the whole application archive by the CLI.
#include <stdint.h>
#include <stddef.h>
#include <string.h>

extern uint32_t bridgerton_generate_v1(const unsigned char *, size_t);
int main(int argc, char **argv) {
    if (argc != 2) return 2;
    return (int)bridgerton_generate_v1((const unsigned char *)argv[1], strlen(argv[1]));
}
