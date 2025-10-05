# rustcoding
A collection of Rust Coding

## Testing using docker and minikube

Here's a summary of all the commands for a complete test cycle:
```bash
# Build the Docker image and make it available to Minikube:
eval $(minikube -p minikube docker-env)
docker build -t is_subsequence-test .
# Run the test Job on Kubernetes:
kubectl apply -f job.yaml
# Check the results:
kubectl logs -l job-name=is-subsequence-test
# Clean up the completed Job:
kubectl delete job is-subsequence-test
```

## Workspace Management

This repository is a Cargo workspace. Recently, the workspace was cleaned up to resolve several issues:

*   Fixed a Cargo workspace configuration issue that caused `multiple workspace roots` errors.
*   Resolved a missing `main` function error in the `top_k_frequent` crate.
*   Standardized the workspace to use the 2024 edition resolver.

## How to Run Code

All commands should be run from the root directory of the project.

### Running a Project

To run a specific project, use the `cargo run -p <project-name>` command. For example:

```bash
cargo run -p merge_two_lists_v2
```

### Running Tests

To run tests for a specific project, use the `cargo test -p <project-name>` command. For example:

```bash
cargo test -p merge_two_lists_v2
```

### Available Projects

Here is a list of the available projects in this workspace:

*   `chatbot`
*   `container_with_most_water`
*   `delete_duplicates`
*   `delete_nodes_linked_list`
*   `hello`
*   `hello_actix`
*   `invert_tree`
*   `is_palindrome`
*   `is_subsequence`
*   `merge_two_lists`
*   `merge_two_lists_v2`
*   `next_greater_element_1`
*   `remove_linked_list_elements`
*   `remove_nodes_from_linked_list`
*   `rust_coding_lib`
*   `same_tree`
*   `top_k_frequent`
*   `two_sum`
*   `valid_parentheses`
*   `zigzag_level_order`
*   `my_utils`
*   `nb_prelude`
