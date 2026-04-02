import { RenderMode, ServerRoute } from '@angular/ssr';

export const serverRoutes: ServerRoute[] = [
    {
    path: 'portfolio/detail/:kind',
    renderMode: RenderMode.Prerender, // This fixes the error by using SSR instead of SSG
    getPrerenderParams: async () => {
      // You can fetch this data from an API using Angular's inject()
      return [
        { kind: 'component' },
      ];
    },
  },
  {
    path: '**',
     renderMode: RenderMode.Server
    // renderMode: RenderMode.Prerender
  }
];
