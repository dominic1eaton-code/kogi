import { Component, OnDestroy, OnInit, inject } from '@angular/core';
import { Router } from '@angular/router';

@Component({
  selector: 'app-creating-workspace',
  standalone: true,
  imports: [],
  templateUrl: './creating-workspace.component.html',
  styleUrl: './creating-workspace.component.css',
  host: {
    class: 'block w-full min-h-screen'
  }
})
export class CreatingWorkspaceComponent implements OnInit, OnDestroy {
  private router = inject(Router);
  private timeoutId: ReturnType<typeof setTimeout> | null = null;

  ngOnInit(): void {
    this.timeoutId = setTimeout(() => {
      this.router.navigate(['/dashboard']);
    }, 2500);
  }

  ngOnDestroy(): void {
    if (this.timeoutId !== null) {
      clearTimeout(this.timeoutId);
    }
  }
}
