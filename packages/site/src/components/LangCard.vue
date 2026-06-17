<template>
    <RouterLink :to="'/lang/' + profile.name.toLowerCase()">
        <div class="card-container">
            <p>
                <strong>{{ profile.name }}</strong> <br>
                <span>{{ profile.description }}</span>
            </p>
        </div>
    </RouterLink>
</template>

<script lang="ts" setup>
import { type LangProfile } from '@/cfg/langs';
const props = defineProps<{ profile: LangProfile }>();
</script>

<style lang="scss" scoped>
@use "@/style/main.scss" as *;

a {
    text-decoration: none;
}

.card-container {
    color: var(--text);
    border-top: 1px var(--border) solid;
    padding: 0 var(--padding-safe);
    display: flow-root;
    box-sizing: border-box;
    background-color: transparent;

    transition: background-color 0.15s;

    strong {
        font-weight: 700;
        transition: font-weight 0.15s; // add this
    }

    p {
        box-sizing: border-box;
    }

    &:hover {
        background-color: var(--surface-1);

        strong {
            font-weight: 800;
        }
    }
}
</style>