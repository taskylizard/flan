<script setup lang="ts">
import { buttonVariants } from '@/components/ui/button'

const username = ref('')
const accessKey = ref('')
const isLoading = ref(false)
const showAlert = ref(false)

const userStore = useUserStore()
const {
  isAuthenticated,
  username: username_,
  accessKey: accessKey_,
} = storeToRefs(userStore)
const { login, logout } = userStore

watch(
  isAuthenticated,
  (newValue) => {
    if (newValue) {
      showAlert.value = true
    }
    else {
      showAlert.value = false
    }
  },
  { immediate: true },
)

async function onSubmit(event: Event) {
  event.preventDefault()
  isLoading.value = true
  login(username.value, accessKey.value)
  isLoading.value = false
}

async function handleLogout() {
  isLoading.value = true
  logout()
  username.value = ''
  accessKey.value = ''
  isLoading.value = false
}

const isDisabled = computed(() => isLoading.value)
</script>

<template>
  <div class="flex min-h-screen items-center justify-center bg-background">
    <Card class="mx-auto w-full max-w-md">
      <CardHeader class="space-y-1">
        <CardTitle class="text-2xl font-bold">
          {{ isAuthenticated ? "Account Settings" : "Login to your account" }}
        </CardTitle>
        <CardDescription v-if="!isAuthenticated">
          Enter your username and access key that was sent to you. Logging in
          saves your credentials in your browser.
        </CardDescription>
        <CardDescription v-else>
          You are currently logged in as
          <code class="relative rounded bg-muted px-[0.3rem] py-[0.2rem] font-mono text-sm font-semibold">@{{ username_
          }}</code>
          and your access key is
          <code class="relative rounded bg-muted px-[0.3rem] py-[0.2rem] font-mono text-sm font-semibold">{{ accessKey_
          }}</code>.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div v-if="!isAuthenticated">
          <form class="space-y-4" @submit="onSubmit">
            <div class="space-y-2">
              <Label for="username">Username</Label>
              <Input
                id="username"
                v-model="username"
                placeholder="taskylizard"
                type="text"
                autocapitalize="none"
                autocorrect="off"
                required
                :disabled="isDisabled"
              />
            </div>
            <div class="space-y-2">
              <Label for="accessKey">Access Key</Label>
              <Input
                id="accessKey"
                v-model="accessKey"
                placeholder="taskylizard-xyz..."
                type="password"
                autocapitalize="none"
                autocorrect="off"
                required
                :disabled="isDisabled"
              />
            </div>
            <Button class="w-full" type="submit" :disabled="isDisabled">
              <ILucideLoader2 v-if="isLoading" class="size-4 animate-spin" />
              Login
            </Button>
          </form>
        </div>
        <div v-else class="flex justify-end space-x-4">
          <RouterLink
            to="/"
            :class="`${buttonVariants({
              variant: 'default',
            })}`"
            class="w-full"
          >
            <ILucideHome class="size-5" />
            Back
          </RouterLink>
          <Button class="w-full" variant="destructive" :disabled="isDisabled" @click="handleLogout">
            <ILucideLoader2 v-if="isLoading" class="mr-2 size-4 animate-spin" />
            <ILucideLogOut v-else class="size-4" />
            Logout
          </Button>
        </div>
      </CardContent>
      <Separator class="my-4" />
      <CardFooter class="text-center text-sm text-muted-foreground">
        <span v-if="!isAuthenticated">
          Don't have an account? You probably shouldn't be here. But, you can
          ask the instance owner to create one for you.
        </span>
        <span v-else>Need help? Contact the instance owner for support.</span>
      </CardFooter>
    </Card>
  </div>
</template>
