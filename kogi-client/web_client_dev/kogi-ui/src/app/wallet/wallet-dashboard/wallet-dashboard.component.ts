import { Component } from '@angular/core';

@Component({
  selector: 'app-wallet-dashboard',
  standalone: true,
  templateUrl: './wallet-dashboard.component.html',
  styleUrl: './wallet-dashboard.component.css',
  host: {
    class: 'block w-full'
  }
})
export class WalletDashboardComponent {}
