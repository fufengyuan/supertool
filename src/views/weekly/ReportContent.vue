<template>
  <div v-if="reportData" class="bg-base-100 rounded-xl p-6 shadow-sm">
    <div class="mb-8 p-4 bg-base-200 rounded-lg">
      <h3 class="m-0 mb-2.5 text-base-content">{{ $t('report.summary') }}</h3>
      <p class="my-1 text-base-content/60">
        {{ $t('report.timeRange') }}: {{ formatDate(reportData.startDate) }} {{ $t('report.to') }} {{ formatDate(reportData.endDate) }}
      </p>
      <p class="my-1 text-base-content/60">{{ $t('report.completedTasks') }}: {{ reportData.completedTasks.length }}</p>
      <p class="my-1 text-base-content/60">{{ $t('report.projectCount') }}: {{ reportData.projects.length }}</p>
    </div>

    <!-- 项目统计表 -->
    <div class="mb-8">
      <h3 class="m-0 mb-4 text-base-content pb-2 border-b-2 border-base-content/10">{{ $t('report.projectStats') }}</h3>
      <table class="table table-zebra w-full rounded-lg overflow-hidden">
        <thead>
          <tr>
            <th>{{ $t('report.project') }}</th>
            <th>{{ $t('report.daysActive') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="projectStat in reportData.projectStats" :key="projectStat.projectId">
            <td>{{ getProjectName(projectStat.projectId) }}</td>
            <td>{{ projectStat.daysActive }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 本周工作表 -->
    <div class="mb-8">
      <h3 class="m-0 mb-4 text-base-content pb-2 border-b-2 border-base-content/10">{{ $t('report.weeklyWork') }}</h3>
      <table class="table table-zebra w-full rounded-lg overflow-hidden">
        <thead>
          <tr>
            <th>{{ $t('report.project') }}</th>
            <th>{{ $t('report.content') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(tasks, projectId) in reportData.weeklyWork" :key="projectId">
            <td>{{ getProjectName(projectId) }}</td>
            <td>
              <ul class="m-0 ps-5">
                <li v-for="task in tasks" :key="task.id" class="mb-1 text-base-content">{{ task.text }}</li>
              </ul>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Git提交记录表 -->
    <div v-if="reportData.gitCommits.length > 0" class="mb-8">
      <h3 class="m-0 mb-4 text-base-content pb-2 border-b-2 border-base-content/10">{{ $t('report.gitCommits') }}</h3>
      <table class="table table-zebra w-full rounded-lg overflow-hidden">
        <thead>
          <tr>
            <th>{{ $t('report.project') }}</th>
            <th>{{ $t('report.commit') }}</th>
            <th>{{ $t('report.author') }}</th>
            <th>{{ $t('report.date') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="commit in reportData.gitCommits" :key="commit.hash">
            <td>{{ commit.projectName }}</td>
            <td>{{ commit.message }}</td>
            <td>{{ commit.author }}</td>
            <td>{{ formatDate(commit.date) }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 下周计划表 -->
    <div class="mb-8">
      <h3 class="m-0 mb-4 text-base-content pb-2 border-b-2 border-base-content/10">{{ $t('report.nextWeekPlan') }}</h3>
      <table class="table table-zebra w-full rounded-lg overflow-hidden">
        <thead>
          <tr>
            <th>{{ $t('report.project') }}</th>
            <th>{{ $t('report.plan') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(tasks, projectId) in reportData.nextWeekPlan" :key="projectId">
            <td>{{ getProjectName(projectId) }}</td>
            <td>
              <ul class="m-0 ps-5">
                <li v-for="task in tasks" :key="task.id" class="mb-1 text-base-content">{{ task.text }}</li>
              </ul>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>

  <div v-else class="text-center p-16 text-base-content/60 text-base bg-base-100 rounded-2xl mt-5">
    <p>{{ $t('report.noReport') }}</p>
  </div>
</template>

<script setup lang="ts">
const props = defineProps({
  reportData: { type: Object, default: null },
  getProjectName: { type: Function, required: true },
  formatDate: { type: Function, required: true },
});
</script>
