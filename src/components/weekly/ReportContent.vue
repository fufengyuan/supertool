<template>
  <div v-if="reportData" class="report-content">
    <div class="report-summary">
      <h3>{{ $t('report.summary') }}</h3>
      <p>
        {{ $t('report.timeRange') }}: {{ formatDate(reportData.startDate) }} {{ $t('report.to') }} {{ formatDate(reportData.endDate) }}
      </p>
      <p>{{ $t('report.completedTasks') }}: {{ reportData.completedTasks.length }}</p>
      <p>{{ $t('report.projectCount') }}: {{ reportData.projects.length }}</p>
    </div>

    <!-- 项目统计表 -->
    <div class="report-section">
      <h3>{{ $t('report.projectStats') }}</h3>
      <table class="report-table">
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
    <div class="report-section">
      <h3>{{ $t('report.weeklyWork') }}</h3>
      <table class="report-table">
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
              <ul class="task-list">
                <li v-for="task in tasks" :key="task.id">{{ task.text }}</li>
              </ul>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Git提交记录表 -->
    <div v-if="reportData.gitCommits.length > 0" class="report-section">
      <h3>{{ $t('report.gitCommits') }}</h3>
      <table class="report-table">
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
    <div class="report-section">
      <h3>{{ $t('report.nextWeekPlan') }}</h3>
      <table class="report-table">
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
              <ul class="task-list">
                <li v-for="task in tasks" :key="task.id">{{ task.text }}</li>
              </ul>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>

  <div v-else class="no-report">
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

<style scoped>
.report-content {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 24px;
  box-shadow: var(--card-shadow);
}

.report-summary {
  margin-bottom: 30px;
  padding: 16px;
  background: var(--input-bg);
  border-radius: 8px;
}

.report-summary h3 {
  margin: 0 0 10px 0;
  color: var(--main-text);
}

.report-summary p {
  margin: 5px 0;
  color: var(--main-text-secondary);
}

.report-section {
  margin-bottom: 30px;
}

.report-section h3 {
  margin: 0 0 15px 0;
  color: var(--main-text);
  padding-bottom: 8px;
  border-bottom: 2px solid var(--border-color);
}

.report-table {
  width: 100%;
  border-collapse: collapse;
  margin-bottom: 20px;
  background: var(--input-bg);
  border-radius: 8px;
  overflow: hidden;
}

.report-table th,
.report-table td {
  padding: 12px;
  text-align: left;
  border-bottom: 1px solid var(--border-color);
}

.report-table th {
  background: var(--primary-color);
  color: white;
  font-weight: 600;
}

.report-table tbody tr:last-child td {
  border-bottom: none;
}

.report-table tbody tr:hover {
  background: var(--card-bg);
}

.task-list {
  margin: 0;
  padding-left: 20px;
}

.task-list li {
  margin-bottom: 5px;
  color: var(--main-text);
}

.no-report {
  text-align: center;
  padding: 60px 20px;
  color: var(--main-text-secondary);
  font-size: 16px;
  background: var(--card-bg);
  border-radius: 16px;
  margin-top: 20px;
}
</style>
