# Ensure all targets run even if a file or directory named these exist
.PHONY: all start-minikube deploy-dummy run-script clean

# 'all' runs the entire flow in sequence:
# 1) Start Minikube
# 2) Create a pod with Python
# 3) Run the Rust script
all: start-minikube deploy-dummy run-script

#######################
# 1) Start minikube
#######################
start-minikube:
	@echo "==> Starting minikube..."
	minikube start

#######################
# 2) Create the Python pod
#######################
deploy-dummy:
	@echo "==> Making sure pod is not up yet..."
	kubectl delete deployment dummy

	@echo "==> Creating a pod that runs a random Python program..."
	# This will run an infinite loop printing random numbers every second.
	kubectl apply -f dummy/deployment.yaml

	@echo "==> Waiting a few seconds for the pod to get scheduled..."
	sleep 5
	kubectl get pods

#######################
# 3) Run the Rust script
#######################
run-script:
	@echo "==> Finding pod name for deployment 'dummy'..."
	@POD_NAME=$$(kubectl get pods | grep Running | awk '{print $$1}') && \
	echo "==> Found pod: $$POD_NAME" && \
	echo "==> Building & running Rust script..." && \
	export RUST_BACKTRACE=1 && cargo build && cargo run $$POD_NAME default 5


#######################
# Utility
#######################
clean:
	@echo "==> Stopping minikube and cleaning up the cluster..."
	minikube stop
