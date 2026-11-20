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
  import { register } from '$lib/auth';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';

  let invite = $state($page.url.searchParams.get('invite') || '');
  let email = $state('');
  let username = $state('');
  let password = $state('');
  let error = $state('');
  let success = $state('');
  let loading = $state(false);

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();

    if (!invite || !email || !username || !password) {
      error = 'Please fill in all fields';
      return;
    }

    error = '';
    success = '';
    loading = true;

    try {
      await register(invite, email, username, password);
      success = 'Registration successful! You can now sign in.';

      setTimeout(() => {
        goto('/auth/sign-in');
      }, 2000);
    } catch (err: any) {
      if (err.error_code) {
        switch (err.error_code) {
          case 'INVALID_INVITE':
            error = 'This invite code is invalid or does not exist.';
            break;
          case 'USED_INVITE':
            error = 'This invite code has already been used.';
            break;
          case 'VALIDATION_FAILED':
            error = err.message || 'Please check your input details.';
            break;
          default:
            error = err.message || 'Registration failed.';
        }
      } else {
        error = err.message || 'Registration failed. Please try again.';
      }
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head>
  <title>Sign Up - Sunder</title>
</svelte:head>

<Card padding={8} width="100%">
  <form onsubmit={handleSubmit}>
    <VStack gap={5}>
      <VStack gap={1}>
        <Text type="supporting" color="secondary">Invite-only access</Text>
        <Heading level={2}>Create account</Heading>
      </VStack>

      {#if error}
        <Banner status="error" title={error} container="card" />
      {/if}

      {#if success}
        <Banner status="success" title={success} container="card" />
      {/if}

      <FormLayout defaultOptionality="required">
        <TextInput
          label="Invite code"
          value={invite}
          onChange={(v) => (invite = v)}
          isDisabled={loading || success !== ''}
          hasAutoFocus={!invite}
          size="lg"
        />
        <TextInput
          label="Email"
          type="email"
          value={email}
          onChange={(v) => (email = v)}
          isDisabled={loading || success !== ''}
          hasAutoFocus={!!invite}
          size="lg"
        />
        <TextInput
          label="Username"
          value={username}
          onChange={(v) => (username = v)}
          isDisabled={loading || success !== ''}
          size="lg"
        />
        <TextInput
          label="Password"
          type="password"
          value={password}
          onChange={(v) => (password = v)}
          isDisabled={loading || success !== ''}
          size="lg"
        />
      </FormLayout>

      <Button
        label={loading ? 'Creating account...' : 'Sign up'}
        type="submit"
        variant="primary"
        size="lg"
        width="100%"
        isLoading={loading}
        isDisabled={success !== ''}
      />

      <Divider />

      <VStack hAlign="center">
        <Text type="supporting" color="secondary">
          Already have an account? <Link href="/auth/sign-in" type="supporting">Sign in</Link>
        </Text>
      </VStack>
    </VStack>
  </form>
</Card>
