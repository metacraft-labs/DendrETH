add_executable(weigh_justification_and_finalization_test weigh_justification_and_finalization_test.cpp)

add_dependencies(template weigh_justification_and_finalization_test)

target_link_libraries (weigh_justification_and_finalization_test
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
LLVMSupport

${Boost_LIBRARIES})

add_test(weigh_justification_and_finalization_test compute_shuffled_index_test)

target_include_directories(weigh_justification_and_finalization_test PRIVATE ${PROJECT_SOURCE_DIR}/src)