#pragma once

#include <array>
#include <cstdint>

#include "circuit_byte_utils.h"

template <typename T, std::size_t CAPACITY>
struct static_vector {

    std::array<T, CAPACITY> content_;
    size_t size_;

    template<std::size_t SIZE>
    constexpr static_vector(std::array<T, SIZE> const & rhs) {
        static_assert(SIZE <= CAPACITY);
        circuit_byte_utils::copy(rhs.begin(), rhs.end(), begin());
        size_ = SIZE;
    }
    constexpr static_vector() {
        size_ = 0;
    }
    constexpr static_vector(static_vector&& rhs) {
        content_ = std::move(rhs);
        size_ = rhs.size_;
    }
    constexpr static_vector(static_vector const & rhs) {
        content_ = rhs;
        size_ = rhs.size_;
    }
    // For some reason, this triggers a circuit compilation error
    // ~static_vector() {
    //     size_ = 0;
    // }
    constexpr auto operator=(static_vector&& rhs) -> static_vector& {
        content_ = std::move(rhs);
        size_ = rhs.size_;
        return *this;
    }
    constexpr auto operator=(static_vector const & rhs) -> static_vector& {
        content_ = std::move(rhs);
        size_ = rhs.size_;
        return *this;
    }
    constexpr auto data() -> T * {
        return &content_;
    }
    constexpr auto operator[](std::size_t index) -> T& {
        return content_[index];
    }
    constexpr auto begin() {
        return content_.begin();
    }
    constexpr auto end() {
        return content_.begin() + size_;
    }
    constexpr auto size() {
        return size_;
    }
    constexpr auto capacity() -> std::size_t {
        return CAPACITY;
    }
    constexpr auto push_back(T const& val) -> void {
        assert_true(size_ < CAPACITY);
        content_[size_++] = val;
    }
    constexpr auto pop_back(T const& val) -> void {
        assert_true(size_ > 0);
        --size_;
    }
};