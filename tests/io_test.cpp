#include <cassert>
#include <iomanip>
#include <sstream>

#include "cp/io.hpp"

int main() {
    {
        std::ostringstream out;
        cp::println_to(out, 10, "abc", -2, 'x');
        assert(out.str() == "10 abc -2 x\n");
    }
    {
        std::ostringstream out;
        cp::println_to(out);
        assert(out.str() == "\n");
    }
    {
        std::ostringstream out;
        cp::println_fixed_to(out, 3, 1.23456, 2.0, -0.1254);
        assert(out.str() == "1.235 2.000 -0.125\n");
    }
    {
        std::ostringstream out;
        out << std::scientific << std::setprecision(1);
        cp::println_fixed_to(out, 2, 1.25);
        out << 1.25;
        assert(out.str() == "1.25\n1.2e+00");
    }
}

