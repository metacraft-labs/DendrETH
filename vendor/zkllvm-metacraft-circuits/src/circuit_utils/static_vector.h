#pragma once

#include <array>
#include <cstdint>

#include "circuit_byte_utils.h"

template<typename T, std::size_t CAPACITY = 128>
struct static_vector {

    T content_[CAPACITY];
    size_t size_;

    template<std::size_t SIZE>
    constexpr explicit static_vector(std::array<T, SIZE> const& rhs) {
        static_assert(SIZE <= CAPACITY);
        for(size_t i = 0; i < SIZE; i++) {
            content_[i] = rhs[i];
        }
        size_ = SIZE;
    }
    constexpr static_vector() {
        size_ = 0;
    }
    constexpr static_vector(static_vector const& rhs) {
        for(size_t i = 0; i < rhs.size_; i++) {
            content_[i] = rhs[i];
        }
        size_ = rhs.size_;
    }
    // For some reason, this triggers a circuit compilation error
    // ~static_vector() {
    //     size_ = 0;
    // }
    constexpr auto operator=(static_vector const& rhs) -> static_vector& {
        for(size_t i = 0; i < rhs.size_; i++) {
            content_[i] = rhs[i];
        }
        size_ = rhs.size_;
        return *this;
    }
    constexpr auto data() -> T* {
        return &content_;
    }
    constexpr auto content() -> std::array<T, CAPACITY>& {
        return reinterpret_cast<std::array<T, CAPACITY>&>(content_);
    }
    constexpr auto operator[](std::size_t index) -> T& {
        return content_[index];
    }
    constexpr auto operator[](std::size_t index) const -> const T& {
        const T& retval = content_[index];
        return retval;
    }
    constexpr auto size() const {
        return size_;
    }
    constexpr auto capacity() -> std::size_t {
        return CAPACITY;
    }
    constexpr auto push_back(T const& val) -> void {
        assert_true(size_ < CAPACITY);
        content_[size_++] = val;
    }
    constexpr void clear() {
        size_ = 0;
    }
    constexpr bool full() const {
        return CAPACITY == size_;
    }
    constexpr auto pop_back(T const& val) -> void {
        assert_true(size_ > 0);
        --size_;
    }
} __attribute__((packed));
