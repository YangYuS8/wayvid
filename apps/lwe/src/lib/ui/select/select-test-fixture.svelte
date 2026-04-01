<script lang="ts">
  import * as Select from './index';
</script>

<Select.Root type="single" value="primary" name="monitor" open={true}>
  <Select.Trigger>Primary monitor</Select.Trigger>
  <Select.Content>
    <Select.Item value="primary" label="Primary monitor">Primary monitor</Select.Item>
  </Select.Content>
</Select.Root>
