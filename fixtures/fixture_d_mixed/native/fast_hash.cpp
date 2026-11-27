#include "fast_hash.hpp"
#include <iostream>

namespace native_hash {
    int compute_checksum(int seed) {
        return seed * 31;
    }
}
