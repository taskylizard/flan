<script setup lang="ts">
import { buttonVariants } from './ui/button'
</script>

<template>
  <header class="sticky top-0 z-40 w-full border-b bg-background">
    <div class="container flex h-16 items-center space-x-4 sm:justify-between sm:space-x-0">
      <div class="flex gap-6 md:gap-10">
        <RouterLink to="/" class="flex items-center space-x-2">
          <INotoRibbon class="size-6 text-foreground" />
        </RouterLink>
      </div>
      <div class="flex flex-1 items-center justify-end space-x-4">
        <nav class="flex items-center space-x-1">
          <a href="https://github.com/taskylizard/flan" target="_blank" rel="noreferrer">
            <div
              :class="`${buttonVariants({
                size: 'icon',
                variant: 'ghost',
              })}`"
            >
              <IRadixGithubLogo class="size-5" />
              <span class="sr-only">GitHub</span>
            </div>
          </a>
          <ThemeToggle />
        </nav>
      </div>
    </div>
  </header>
</template>
