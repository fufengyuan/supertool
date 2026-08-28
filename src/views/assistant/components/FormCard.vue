<template>
  <div class="border rounded-xl overflow-hidden bg-base-100 border-base-content/10">
    <div class="flex items-center gap-2 px-3 py-2 bg-base-200 border-b border-base-content/10">
      <SvgIcon name="clipboard" size="14" class="shrink-0 text-primary" />
      <span class="text-xs font-semibold text-base-content truncate flex-1">{{ form.title }}</span>
      <span class="text-[10px] px-1.5 py-0.5 rounded-full shrink-0" :class="badgeClass">{{ statusLabel }}</span>
    </div>

    <div class="p-3 flex flex-col gap-2.5">
      <p v-if="form.description" class="text-[11px] text-base-content/70 m-0 leading-relaxed">
        {{ form.description }}
      </p>

      <template v-if="form.status === 'pending'">
        <div v-for="f in form.fields" :key="f.name" class="flex flex-col gap-1">
          <label class="text-[11px] text-base-content/75 flex items-center gap-1">
            {{ f.label }}
            <span v-if="f.required" class="text-error">*</span>
            <span v-if="f.type === 'password'" class="text-warning/80 flex items-center gap-0.5">
              <SvgIcon name="lock" size="10" /> 仅保存在本地，不进对话
            </span>
          </label>

          <input
            v-if="f.type === 'text' || f.type === 'password'"
            v-model="draft[f.name]"
            :type="f.type === 'password' ? 'password' : 'text'"
            :placeholder="f.placeholder"
            autocomplete="new-password"
            class="input input-bordered input-xs w-full text-[11px]"
          />
          <input
            v-else-if="f.type === 'number'"
            v-model.number="draft[f.name]"
            type="number"
            :placeholder="f.placeholder"
            class="input input-bordered input-xs w-40 font-mono text-[11px]"
          />
          <select
            v-else-if="f.type === 'select'"
            v-model="draft[f.name]"
            class="select select-bordered select-xs w-full text-[11px]"
          >
            <option value="" disabled>请选择</option>
            <option v-for="o in f.options" :key="o" :value="o">{{ o }}</option>
          </select>
          <label v-else-if="f.type === 'boolean'" class="flex items-center gap-1.5 cursor-pointer">
            <input v-model="draft[f.name]" type="checkbox" class="toggle toggle-xs toggle-primary" />
            <span class="font-mono text-[11px]">{{ draft[f.name] ? '是' : '否' }}</span>
          </label>
          <textarea
            v-else
            v-model="draft[f.name]"
            rows="2"
            :placeholder="f.placeholder"
            class="textarea textarea-bordered w-full text-[11px] leading-snug"
          ></textarea>

          <p v-if="f.description" class="text-[10px] text-base-content/45 m-0">{{ f.description }}</p>
        </div>

        <p v-if="localError" class="text-[11px] text-error m-0">{{ localError }}</p>

        <div class="flex items-center gap-2 pt-0.5">
          <button class="btn btn-primary btn-xs" @click="submit">
            <SvgIcon name="check" size="12" /> 提交
          </button>
        </div>
      </template>

      <p v-else class="text-[11px] text-success flex items-center gap-1 m-0">
        <SvgIcon name="check" size="12" /> 已提交，助手正在处理
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import SvgIcon from '../../../components/ui/SvgIcon.vue'
import type { AssistantForm } from '../../../composables/useAssistantChat'

const props = defineProps<{ form: AssistantForm }>()
const emit = defineEmits<{ (e: 'submit', values: Record<string, unknown>): void }>()

const localError = ref('')
const draft = reactive<Record<string, any>>({})

for (const f of props.form.fields || []) {
  // 密码类字段不留默认值；布尔给 false；其余用默认值或空串
  draft[f.name] = f.type === 'password' ? '' : f.default !== undefined ? f.default : f.type === 'boolean' ? false : ''
}

const statusLabel = computed(() => (props.form.status === 'submitted' ? '已提交' : '待填写'))
const badgeClass = computed(() =>
  props.form.status === 'submitted' ? 'bg-success/10 text-success' : 'bg-primary/10 text-primary',
)

function submit() {
  for (const f of props.form.fields || []) {
    if (!f.required) {continue}
    const v = draft[f.name]
    if (v === undefined || v === null || v === '') {
      localError.value = `「${f.label}」是必填项`
      return
    }
  }
  localError.value = ''
  emit('submit', { ...draft })
}
</script>
