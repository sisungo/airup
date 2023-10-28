#include <airup_sdk/config.hpp>
#include <rapidjson/document.h>

airup::config::build_manifest& airup::config::build_manifest::get() {
    if (airup::config::build_manifest::singleton_instance.has_value()) {
        return airup::config::build_manifest::singleton_instance.value();
    }

    constexpr auto& raw_text = R"(
#include "../../build_manifest.json"
    )";

    rapidjson::Document document;

    document.Parse(raw_text);
    if (document.HasParseError()) {
        throw airup::config::build_manifest_exception();
    }
    airup::config::build_manifest object;
    object.os_name = document["os_name"].GetString();
    object.config_dir = document["config_dir"].GetString();
    object.service_dir = document["service_dir"].GetString();
    object.milestone_dir = document["milestone_dir"].GetString();
    object.runtime_dir = document["runtime_dir"].GetString();
    object.log_dir = document["log_dir"].GetString();
    
    auto env_vars_node = document["env_vars"].GetObject();
    for (auto& env_var : env_vars_node) {
        object.env_vars.insert(std::pair(env_var.name.GetString(), env_var.value.GetString()));
    }

    auto early_cmds_array = document["early_cmds"].GetArray();
    for (auto& early_cmd : early_cmds_array) {
        object.early_cmds.push_back(early_cmd.GetString());
    }

    auto security = document["security"].GetString();
    if (strcmp(security, "policy") == 0) {
        object.security = airup::config::security_model::policy;
    } else if (strcmp(security, "simple") == 0) {
        object.security = airup::config::security_model::simple;
    } else if (strcmp(security, "disabled") == 0) {
        object.security = airup::config::security_model::disabled;
    } else {
        throw airup::config::build_manifest_exception();
    }

    airup::config::build_manifest::singleton_instance = object;
    return airup::config::build_manifest::singleton_instance.value();
}