import { CommonModule } from '@angular/common';
import { Component, OnDestroy, OnInit, inject } from '@angular/core';
import { ActivatedRoute, RouterLink } from '@angular/router';
import { Subscription } from 'rxjs';
import { NavigationComponent } from '../../index/navigation/navigation.component';

@Component({
  selector: 'app-portfolio-create',
  standalone: true,
  imports: [CommonModule, RouterLink, NavigationComponent],
  templateUrl: './portfolio-create.component.html',
  styleUrl: './portfolio-create.component.css',
  host: {
    class: 'block w-full min-h-screen'
  }
})
export class PortfolioCreateComponent implements OnInit, OnDestroy {
  viewMode: 'item' | 'container' | 'all' = 'all';
  activeFragment: 'item' | 'container' | 'review' = 'item';

  private readonly route = inject(ActivatedRoute);
  private fragmentSub?: Subscription;

  get showItem(): boolean {
    return this.viewMode !== 'container';
  }

  get showContainer(): boolean {
    return this.viewMode !== 'item';
  }

  get showReview(): boolean {
    return this.viewMode === 'all';
  }

  get itemNavClass(): string {
    return this.activeFragment === 'item' ? this.activeNavClass : this.inactiveNavClass;
  }

  get containerNavClass(): string {
    return this.activeFragment === 'container' ? this.activeNavClass : this.inactiveNavClass;
  }

  get reviewNavClass(): string {
    return this.activeFragment === 'review' ? this.activeNavClass : this.inactiveNavClass;
  }

  private readonly activeNavClass =
    'flex items-center gap-2 rounded-[8px] bg-[rgba(16,185,129,0.15)] px-2.5 py-1.5 text-[#10b981]';
  private readonly inactiveNavClass =
    'flex items-center gap-2 rounded-[8px] px-2.5 py-1.5 text-[#8ea6ad] hover:bg-[#132830] hover:text-[#e6f1f4]';

  ngOnInit(): void {
    this.fragmentSub = this.route.fragment.subscribe((fragment) => {
      if (fragment === 'item') {
        this.viewMode = 'item';
        this.activeFragment = 'item';
        return;
      }

      if (fragment === 'container') {
        this.viewMode = 'container';
        this.activeFragment = 'container';
        return;
      }

      if (fragment === 'review') {
        this.viewMode = 'all';
        this.activeFragment = 'review';
        return;
      }

      this.viewMode = 'all';
      this.activeFragment = 'item';
    });
  }

  ngOnDestroy(): void {
    this.fragmentSub?.unsubscribe();
  }
}
