set(CMAKE_BUILD_TYPE "Debug")

add_compile_options(-fsanitize=address)
add_link_options(-fsanitize=address)

function(add_example example_target)
    set(prefix ARG)
    set(noValues "")
    set(singleValues)
    set(multiValues SOURCES)
    cmake_parse_arguments(${prefix}
                        "${noValues}"
                        "${singleValues}"
                        "${multiValues}"
                        ${ARGN})
    add_circuit(${example_target}
                SOURCES ${ARG_SOURCES}

                LINK_LIBRARIES
                crypto3::algebra
                crypto3::block
                crypto3::codec
                crypto3::containers
                crypto3::hash
                crypto3::kdf
                crypto3::mac
                marshalling::core
                marshalling::crypto3_algebra
                marshalling::crypto3_multiprecision
                marshalling::crypto3_zk
                crypto3::math
                crypto3::modes
                crypto3::multiprecision
                crypto3::passhash
                crypto3::pbkdf
                crypto3::threshold
                crypto3::pkpad
                crypto3::pubkey
                crypto3::random
                crypto3::stream
                crypto3::vdf
                crypto3::zk

                ${Boost_LIBRARIES})
endfunction()

cm_find_package(Boost COMPONENTS REQUIRED filesystem)

find_package(yaml-cpp REQUIRED)

add_custom_target(template)

# enable testing functionality
enable_testing()

add_subdirectory(tests)
add_subdirectory(circuits)
add_subdirectory(circuit_input_generators)
