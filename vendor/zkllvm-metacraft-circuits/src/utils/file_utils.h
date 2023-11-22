#include "boost/filesystem.hpp"
using namespace boost::filesystem;

namespace file_utils {
    struct non_existing_path : public std::runtime_error {
        template <typename T>
        non_existing_path(T&& arg) : runtime_error(std::forward<T>(arg)) {}
    };

    void find_matching_files(const path& dir_path,                        // in this directory,
                             const std::vector<std::string>& patterns,    // search for this name,
                             std::vector<path>& path_found)               // placing path here if found
    {
        auto check_matching = [](const std::string& file_path, const std::vector<std::string>& patterns) {
            return std::all_of(patterns.begin(), patterns.end(), [&file_path](const std::string& pattern) { return file_path.find(pattern) != std::string::npos; });
        };

        if (!exists(dir_path))
            throw non_existing_path(dir_path.string());
        
        directory_iterator end_itr;    // default construction yields past-the-end
        for (directory_iterator itr(dir_path); itr != end_itr; ++itr) {
            if (is_directory(itr->status())) {
                find_matching_files(itr->path(), patterns, path_found);
            } else if (check_matching(itr->path().string(), patterns)) {
                path_found.push_back(itr->path());
            }
        }
    }
}    // namespace file_utils
