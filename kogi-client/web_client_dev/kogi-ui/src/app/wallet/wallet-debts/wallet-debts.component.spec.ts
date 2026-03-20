import { ComponentFixture, TestBed } from '@angular/core/testing';
import { WalletDebtsComponent } from './wallet-debts.component';

describe('WalletDebtsComponent', () => {
  let component: WalletDebtsComponent;
  let fixture: ComponentFixture<WalletDebtsComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [WalletDebtsComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(WalletDebtsComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
