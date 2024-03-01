#pragma once

#include <array>
#include <cstdint>
#include <initializer_list>

using Byte = unsigned char;

template<typename T, std::size_t Capacity = 128, bool InitiallyFull = false>
struct static_vector {

    static constexpr auto capacity = Capacity;

    size_t size_;
    T content_[Capacity];

    constexpr static_vector(std::initializer_list<T> init) {
        size_ = 0;
        for (const auto& v : init) {
            assert_true(size_ < Capacity);
            content_[size_++] = v;
        }
    }

    template<std::size_t Size>
    constexpr explicit static_vector(const std::array<T, Size>& rhs) {
        static_assert(Size <= Capacity);
        for (size_t i = 0; i < Size; i++) {
            content_[i] = rhs[i];
        }
        size_ = Size;
    }
    template<std::size_t Size, bool Full>
    constexpr explicit static_vector(const static_vector<T, Size, Full>& rhs) {
        static_assert(Size <= Capacity);
        for (size_t i = 0; i < Size; i++) {
            content_[i] = rhs[i];
        }
        size_ = Size;
    }
    constexpr static_vector() {
        if constexpr (InitiallyFull) {
            size_ = Capacity;
        } else {
            size_ = 0;
        }
    }
    // For some reason, this triggers a circuit compilation error
    // ~static_vector() {
    //     size_ = 0;
    // }
    constexpr auto data() -> T* {
        return &content_;
    }
    constexpr auto content() -> std::array<T, Capacity>& {
        return reinterpret_cast<std::array<T, Capacity>&>(content_);
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
    constexpr auto push_back(T const& val) -> void {
        assert_true(size_ < Capacity);
        content_[size_++] = val;
    }
    constexpr void clear() {
        size_ = 0;
    }
    constexpr bool full() const {
        return Capacity == size_;
    }
    constexpr auto pop_back(T const& val) -> void {
        assert_true(size_ > 0);
        --size_;
    }
    bool operator==(const static_vector& rhs) const {
        if (size_ != rhs.size_) {
            return false;
        }
        for (size_t i = 0; i < size_; i++) {
            if (content_[i] != rhs[i]) {
                return false;
            }
        }
        return true;
    }
    bool operator!=(const static_vector& rhs) const {
        return !(*this == rhs);
    }
} __attribute__((packed));

template<std::size_t Capacity>
struct static_vector<Byte, Capacity> {

    static constexpr auto capacity = Capacity;

    std::array<Byte, Capacity> content_;

    constexpr static_vector(std::initializer_list<Byte> init) {
        size_t i = 0;
        for (const auto& v : init) {
            assert_true(i < Capacity);
            content_[i++] = v;
        }
    }

    template<std::size_t Size>
    constexpr explicit static_vector(std::array<Byte, Size> const& rhs) {
        static_assert(Size <= Capacity);
        for (size_t i = 0; i < Size; i++) {
            content_[i] = rhs[i];
        }
    }
    template<std::size_t Size>
    constexpr explicit static_vector(static_vector<Byte, Size> const& rhs) {
        static_assert(Size <= Capacity);
        for (size_t i = 0; i < Size; i++) {
            content_[i] = rhs[i];
        }
    }
    constexpr static_vector() {
        for (size_t i = 0; i < Capacity; i++) {
            content_[i] = 0;
        }
    }

    // For some reason, this triggers a circuit compilation error
    // ~static_vector() {
    //     size_ = 0;
    // }
    constexpr auto operator=(static_vector const& rhs) -> static_vector& {
        for (size_t i = 0; i < Capacity; i++) {
            content_[i] = rhs[i];
        }
        return *this;
    }
    constexpr auto data() -> Byte* {
        return &content_;
    }
    constexpr auto content() -> std::array<Byte, Capacity>& {
        return content_;
    }
    constexpr auto operator[](std::size_t index) -> Byte& {
        return content_[index];
    }
    constexpr auto operator[](std::size_t index) const -> const Byte& {
        const Byte& retval = content_[index];
        return retval;
    }
    constexpr auto size() const -> std::size_t {
        return Capacity;
    }
    bool operator==(const static_vector& rhs) const {
        for (size_t i = 0; i < Capacity; i++) {
            if (content_[i] != rhs[i]) {
                return false;
            }
        }
        return true;
    }
    bool operator!=(const static_vector& rhs) const {
        return !(*this == rhs);
    }
    constexpr auto begin() {
        return &content_[0];
    }
    constexpr auto end() {
        return &content_[0] + Capacity;
    }
    constexpr const auto begin() const {
        return &content_[0];
    }
    constexpr const auto end() const {
        return &content_[0] + Capacity;
    }
} __attribute__((packed));

using Bytes32 = static_vector<Byte, 32>;
using Bytes48 = static_vector<Byte, 48>;
using Bytes64 = static_vector<Byte, 64>;
using Bytes96 = static_vector<Byte, 96>;
