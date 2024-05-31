[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

# Restrict StorageClasses in PersistentVolumeClaims

This Kubewarden policy is designed to enhance the security and manageability of
Kubernetes clusters by preventing the use of certain storage classes within
`PersistentVolumeClaim` (PVC) objects. The policy provides an option to
configure a fallback storage class, offering a seamless alternative when a
denied storage class is requested.

## Configuration

The policy is configurable to meet the needs of different Kubernetes
environments. Below is the structure of the policy's configuration parameters:

```yaml
# List of storage classes that are not allowed
deniedStorageClasses:
- fast
- nvme

# Optional: Specifies the fallback storage class to use when a denied storage class is requested
fallbackStorageClass: slow
```

The fallback storage class is optional. If not specified, the policy will
reject. Furthermore, the `fallbackStorageClass` values cannot be defined in the
`deniedStorageClasses` list.

## How It Works

The policy operates by evaluating the `storageClassName` specified in
`PersistentVolumeClaim` objects. If a PVC requests a storage class listed in
`deniedStorageClasses`, the policy action will depend on the configuration:

- Without a `fallbackStorageClass` specified, the PVC will be rejected.
- With a `fallbackStorageClass` specified, the PVC will be mutated to use the
  fallback storage class, allowing the request to proceed.

## Examples

### Example 1: Rejecting a Denied Storage Class

Given the configuration:

```yaml
deniedStorageClasses:
- fast
```

A PVC with `storageClassName: fast` will be rejected:

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: task-pv-claim
spec:
  storageClassName: fast
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 3Gi
```

### Example 2: Accepting and Mutating to Fallback Storage Class

With the following configuration:

```yaml
deniedStorageClasses:
- fast
fallbackStorageClass: cheap
```

A PVC requesting a denied storage class will be mutated to use the fallback class `cheap`, thus being accepted:

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: task-pv-claim
spec:
  storageClassName: fast
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 3Gi
```

Will be mutated to:

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: task-pv-claim
spec:
  storageClassName: cheap
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 3Gi
```
