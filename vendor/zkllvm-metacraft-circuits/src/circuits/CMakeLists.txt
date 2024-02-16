add_example(compute_shuffled_index SOURCES compute_shuffled_index.cpp INPUT main.inp)
add_example(weigh_justification_and_finalization SOURCES weigh_justification_and_finalization.cpp INPUT main.inp)

add_dependencies(template compute_shuffled_index)
add_dependencies(template weigh_justification_and_finalization)

add_custom_command(TARGET compute_shuffled_index POST_BUILD
    COMMAND cp -r ${CMAKE_CURRENT_SOURCE_DIR}/compute_shuffled_index.json ${CMAKE_CURRENT_BINARY_DIR}
    DEPENDS compile_commands.json
    VERBATIM ON
)
