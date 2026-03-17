import { Component } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';

@Component({
  selector: 'app-wallet-wallets',
  standalone: true,
  imports: [RouterLink, RouterLinkActive, RouterOutlet],
  templateUrl: './wallet-wallets.component.html',
  styleUrl: './wallet-wallets.component.css',
  host: {
    class: 'block w-full'
  }
})
export class WalletWalletsComponent {}
