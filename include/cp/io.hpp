#pragma once

#include <cstddef>
#include <iomanip>
#include <iostream>
#include <limits>
#include <ostream>
#include <stdexcept>

namespace cp {
namespace detail {

// Keep formatting helpers from unexpectedly changing the caller's stream.
class stream_state_guard {
public:
    explicit stream_state_guard(std::ostream& out)
        : out_(out), flags_(out.flags()), precision_(out.precision()) {}

    ~stream_state_guard() {
        out_.flags(flags_);
        out_.precision(precision_);
    }

    stream_state_guard(const stream_state_guard&) = delete;
    stream_state_guard& operator=(const stream_state_guard&) = delete;

private:
    std::ostream& out_;
    std::ios_base::fmtflags flags_;
    std::streamsize precision_;
};

template <class First, class... Rest>
void write_space_separated(std::ostream& out, const First& first,
                           const Rest&... rest) {
    out << first;
    ((out << ' ' << rest), ...);
}

inline void write_space_separated(std::ostream&) {}

inline void set_fixed_precision(std::ostream& out, std::size_t digits) {
    const auto max_precision = static_cast<std::size_t>(
        (std::numeric_limits<std::streamsize>::max)());
    if (digits > max_precision) {
        throw std::out_of_range("precision is too large");
    }
    out << std::fixed << std::setprecision(static_cast<std::streamsize>(digits));
}

}  // namespace detail

/** Write all values separated by one ASCII space. */
template <class... Args>
void print_to(std::ostream& out, const Args&... args) {
    detail::write_space_separated(out, args...);
}

/** Write all values separated by spaces, followed by a newline. */
template <class... Args>
void println_to(std::ostream& out, const Args&... args) {
    print_to(out, args...);
    out << '\n';
}

template <class... Args>
void print(const Args&... args) {
    print_to(std::cout, args...);
}

template <class... Args>
void println(const Args&... args) {
    println_to(std::cout, args...);
}

/**
 * Write values in fixed-point notation with `digits` digits after the decimal
 * point. The stream's original flags and precision are restored afterwards.
 */
template <class... Args>
void print_fixed_to(std::ostream& out, std::size_t digits,
                    const Args&... args) {
    detail::stream_state_guard guard(out);
    detail::set_fixed_precision(out, digits);
    print_to(out, args...);
}

template <class... Args>
void println_fixed_to(std::ostream& out, std::size_t digits,
                      const Args&... args) {
    detail::stream_state_guard guard(out);
    detail::set_fixed_precision(out, digits);
    println_to(out, args...);
}

template <class... Args>
void print_fixed(std::size_t digits, const Args&... args) {
    print_fixed_to(std::cout, digits, args...);
}

template <class... Args>
void println_fixed(std::size_t digits, const Args&... args) {
    println_fixed_to(std::cout, digits, args...);
}

}  // namespace cp
