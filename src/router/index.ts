import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'Home',
      component: () => import('../views/Home.vue'),
    },
    {
      path: '/indexer',
      name: 'Indexer',
      component: () => import('../views/indexer/index.vue'),
    },
    {
      path: '/data',
      name: 'Data',
      component: () => import('../views/data/index.vue'),
    },
    {
      path: '/setting',
      name: 'Setting',
      component: () => import('../views/setting/index.vue'),
    },
  ],
})

export default router
