#include <kessler.h>

#include <cstdio>

int main() {
    kessler::Fragments debris;
    // All inputs and outputs are SI (metres, kilograms, metres per second).
    if (kessler_explosion(839.0f, KESSLER_ROCKET_BODY, 0.1f, 7, debris.out()) != KESSLER_STATUS_OK) {
        std::fprintf(stderr, "invalid input\n");
        return 1;
    }

    double total = 0;
    for (size_t i = 0; i < debris->len; ++i)
        total += debris->mass[i];
    std::printf("%zu fragments, %.1f kg\n", debris->len, total);

    kessler::Fragments bad;
    return kessler_explosion(-1.0f, KESSLER_ROCKET_BODY, 0.1f, 7, bad.out()) == KESSLER_STATUS_INVALID_MASS ? 0 : 1;
}
