#include <kessler.h>

#include <cstdio>

int main() {
    kessler::Fragments debris;
    // All inputs and outputs are SI (metres, kilograms, metres per second).
    if (kessler::explosion(839.0f, kessler::SatKind::RocketBody, 0.1f, 7, debris) != kessler::Status::Ok) {
        std::fprintf(stderr, "invalid input\n");
        return 1;
    }

    double total = 0;
    for (size_t i = 0; i < debris->len; ++i)
        total += debris->mass[i];
    std::printf("%zu fragments, %.1f kg\n", debris->len, total);

    auto status = kessler::collision(500.0f, kessler::SatKind::Spacecraft, 5.0f, kessler::SatKind::Spacecraft, 0.0f,
                                     0.1f, 7, debris);
    return status == kessler::Status::InvalidImpactSpeed && debris->len == 0 ? 0 : 1;
}
