import { NgClass, NgIf } from '@angular/common';
import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';
import { WalletDashboardComponent } from './wallet-dashboard/wallet-dashboard.component';

@Component({
  selector: 'app-wallet',
  standalone: true,
  imports: [NgClass, NgIf, RouterLink, WalletDashboardComponent],
  templateUrl: './wallet.component.html',
  styleUrl: './wallet.component.css',
  host: {
    class: 'block w-full min-h-screen'
  }
})
export class WalletComponent {
  activeWalletView: 'wallets' | 'dashboard' = 'wallets';

  readonly activeTopTabClass = 'border-[#10b981] text-[#10b981]';
  readonly inactiveTopTabClass = 'border-transparent text-[#5d747c] hover:text-[#e6f1f4]';

  setActiveWalletView(view: 'wallets' | 'dashboard'): void {
    this.activeWalletView = view;
  }
}
