<template>
  <div class="skeleton-card card">
    <div class="skeleton-card-header">
      <div class="skeleton-circle"></div>
      <div class="skeleton-card-title">
        <div class="skeleton-line skeleton-text" style="width: 60%"></div>
      </div>
    </div>
    <div class="skeleton-card-body">
      <Skeleton variant="text" :rows="2" :line-height="14" last-line-width="80%" />
    </div>
    <div class="skeleton-card-footer">
      <div class="skeleton-badge"></div>
      <div class="skeleton-badge"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import Skeleton from './Skeleton.vue';
</script>

<style scoped>
.skeleton-card {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.skeleton-card-header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.skeleton-circle {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  flex-shrink: 0;
  background: linear-gradient(
    90deg,
    color-mix(in oklab, var(--color-base-content) 10%, transparent) 25%,
    var(--color-base-100) 50%,
    color-mix(in oklab, var(--color-base-content) 10%, transparent) 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s ease-in-out infinite;
}

.skeleton-card-title {
  flex: 1;
}

.skeleton-line {
  border-radius: 6px;
  background: linear-gradient(
    90deg,
    color-mix(in oklab, var(--color-base-content) 10%, transparent) 25%,
    var(--color-base-100) 50%,
    color-mix(in oklab, var(--color-base-content) 10%, transparent) 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s ease-in-out infinite;
}

.skeleton-text {
  height: 14px;
}

.skeleton-card-body {
  padding: 4px 0;
}

.skeleton-card-footer {
  display: flex;
  gap: 8px;
  padding-top: 8px;
  border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.skeleton-badge {
  width: 60px;
  height: 20px;
  border-radius: 10px;
  background: linear-gradient(
    90deg,
    color-mix(in oklab, var(--color-base-content) 10%, transparent) 25%,
    var(--color-base-100) 50%,
    color-mix(in oklab, var(--color-base-content) 10%, transparent) 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s ease-in-out infinite;
}
</style>
