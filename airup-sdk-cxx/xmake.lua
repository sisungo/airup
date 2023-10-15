add_rules("mode.debug", "mode.release")
set_languages("c17", "c++20")

target("airup_sdk")
    set_kind("shared")
    add_files("src/*.cxx")