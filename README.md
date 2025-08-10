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