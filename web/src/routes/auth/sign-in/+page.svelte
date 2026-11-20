<script lang="ts">
  import {
    Banner,
    Button,
    Card,
    Divider,
    FormLayout,
    Heading,
    Link,
    Text,
    TextInput,
    VStack
  } from '@astryx-sv/core';
  import { login } from '$lib/auth';
  import { syncAll } from '$lib/sync';
  import { goto } from '$app/navigation';

  let loginField = $state('');
  let password = $state('');
  let error = $state('');
  let loading = $state(false);

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();

    if (!loginField || !password) {
      error = 'Please fill in all fields';
      return;
    }

    error = '';
    loading = true;

    try {
      await login(loginField, password);
      try {
        await syncAll();
      } catch (syncErr) {
        console.error('Initial sync after login failed:', syncErr);
      }
      goto('/dashboard');
    } catch (err: any) {
      error = err.message || 'Login failed. Please try again.';
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head>
  <title>Sign In - Sunder</title>
</svelte:head>

<Card padding={8} width="100%">
  <form onsubmit={handleSubmit}>
    <VStack gap={5}>
      <VStack gap={1}>
        <Text type="supporting" color="secondary">Welcome back</Text>
        <Heading level={2}>Sign in</Heading>
      </VStack>

      {#if error}
        <Banner status="error" title={error} container="card" />
      {/if}

      <FormLayout defaultOptionality="required">
        <TextInput
          label="Email or username"
          value={loginField}
          onChange={(v) => (loginField = v)}
          isDisabled={loading}
          hasAutoFocus
          size="lg"
        />
        <TextInput
          label="Password"
          type="password"
          value={password}
          onChange={(v) => (password = v)}
          isDisabled={loading}
          size="lg"
        />
      </FormLayout>

      <Button
        label={loading ? 'Signing in...' : 'Sign in'}
        type="submit"
        variant="primary"
        size="lg"
        width="100%"
        isLoading={loading}
      />

      <Divider />

      <VStack hAlign="center">
        <Text type="supporting" color="secondary">
          Don't have an account? <Link href="/auth/sign-up" type="supporting">Sign up</Link>
        </Text>
      </VStack>
    </VStack>
  </form>
</Card>
