add_executable(verify_attestation_data_input_generators verify_attestation_data_input_generators.cpp)

add_dependencies(template verify_attestation_data_input_generators)

target_link_libraries (verify_attestation_data_input_generators
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

target_include_directories(verify_attestation_data_input_generators PRIVATE ${PROJECT_SOURCE_DIR}/src)

add_custom_command(TARGET verify_attestation_data_input_generators POST_BUILD
    COMMAND ${CMAKE_CURRENT_BINARY_DIR}/verify_attestation_data_input_generators ${CMAKE_CURRENT_BINARY_DIR}/../circuits/verify_attestation_data
    DEPENDS compile_commands.json
    VERBATIM ON
)
