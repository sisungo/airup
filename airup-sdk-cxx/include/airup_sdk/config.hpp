#pragma once

#include <string>
#include <vector>
#include <span>
#include <unordered_map>
#include <optional>

#if !defined(__AIRUP_SDK_CONFIG_HPP)
#define __AIRUP_SDK_CONFIG_HPP

namespace airup::config {
    enum class security_model {
        policy,
        simple,
        disabled,
    };

    class build_manifest {
        private:
            static std::optional<build_manifest> singleton_instance;
        public:
            std::string os_name;
            std::string config_dir;
            std::string service_dir;
            std::string milestone_dir;
            std::string runtime_dir;
            std::string log_dir;
            std::unordered_map<std::string, std::string> env_vars;
            std::vector<std::string> early_cmds;
            security_model security;

            static build_manifest& get();
    };

    class build_manifest_exception: std::exception {
        public:
            const char* what() const throw() {
                return "this build of `libairup_sdk` is corrupted because it contains ill-formed build manifest";
            }
    };
}

#endif /* __AIRUP_SDK_CONFIG_HPP */