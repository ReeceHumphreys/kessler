#include <kessler.h>

#include <stdio.h>

int main(void) {
    KesslerFragments debris = {0};
    /* All inputs and outputs are SI (metres, kilograms, metres per second). */
    if (kessler_explosion(839.0f, KESSLER_ROCKET_BODY, 0.1f, 7, &debris) != KESSLER_STATUS_OK) {
        fprintf(stderr, "invalid input\n");
        return 1;
    }

    double total = 0;
    for (size_t i = 0; i < debris.len; ++i)
        total += debris.mass[i];
    printf("%zu fragments, %.1f kg\n", debris.len, total);
    kessler_fragments_free(&debris);

    KesslerStatus status = kessler_collision(500.0f, KESSLER_SPACECRAFT, 5.0f, 9, 9500.0f, 0.1f, 7, &debris);
    return status == KESSLER_STATUS_INVALID_KIND && debris.len == 0 ? 0 : 1;
}
