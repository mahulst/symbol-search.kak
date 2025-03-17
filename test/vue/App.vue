<template>
  <div>
    <h1>{{ title }}</h1>
    <p>{{ message }}</p>
    <button @click="handleClick">Click me</button>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

interface Props {
  title: string
}

type ButtonVariant = 'primary' | 'secondary'

const message = ref('Hello World')

function handleClick() {
  console.log('clicked')
}

const submitForm = (data: Props) => {
  console.log(data)
}
</script>

<style scoped>
.title {
  color: red;
}
</style>
